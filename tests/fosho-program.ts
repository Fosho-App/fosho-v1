import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FoshoProgram } from "../target/types/fosho_program";
import crypto from "crypto";
import { assert } from "chai";
import { createUmi as basecreateUmi } from "@metaplex-foundation/umi-bundle-tests";
import {
  fetchAssetV1,
  fetchCollectionV1,
  mplCore,
} from "@metaplex-foundation/mpl-core";
import { publicKey } from "@metaplex-foundation/umi";

const sleep = (ms: number) => require("timers/promises").setTimeout(ms);

export function createKnownTestKeypair(knownKey: string) {
  try {
    const deterministicSalt = crypto
      .createHash("sha256")
      .update(knownKey)
      .digest();

    const newKey = anchor.web3.Keypair.fromSeed(
      new Uint8Array(deterministicSalt)
    );
    // console.log(knownKey, "Public Key:", newKey.publicKey.toString());
    return newKey;
  } catch (error) {
    throw new Error(`Failed to create keypair: ${error}`);
  }
}

describe("fosho-program", () => {
  // Configure the client to use the local cluster.
  const createUmi = async () => (await basecreateUmi()).use(mplCore());
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.FoshoProgram as Program<FoshoProgram>;

  const seed = anchor.web3.Keypair.generate().publicKey;

  const [community] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("community"), seed.toBuffer()],
    program.programId
  );

  // console.log("Community pubkey", community.toString());

  const eventAuthority = createKnownTestKeypair("eventAuthority");
  const eventAttendee1 = createKnownTestKeypair("eventAttendee1");
  const eventAttendee2 = createKnownTestKeypair("eventAttendee2");
  const eventAttendeeRejected = createKnownTestKeypair("eventAttendeeRejected");

  const getEvent = (nonce: number) => {
    const [event] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("event"),
        community.toBuffer(),
        new anchor.BN(nonce).toArrayLike(Buffer, "le", 4),
      ],
      program.programId
    );

    return event;
  };

  const getAttendeeRecord = (
    event: anchor.web3.PublicKey,
    eventAttendee: anchor.web3.PublicKey
  ) => {
    const [attendeeRecord] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("attendee"), event.toBuffer(), eventAttendee.toBuffer()],
      program.programId
    );

    return attendeeRecord;
  };

  const getEventCollection = (event: anchor.web3.PublicKey) => {
    const [attendeeRecord] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("event"), event.toBuffer(), Buffer.from("collection")],
      program.programId
    );

    return attendeeRecord;
  };

  const getEventTicketAsset = (
    event: anchor.web3.PublicKey,
    owner: anchor.web3.PublicKey
  ) => {
    const [attendeeRecord] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("event"),
        event.toBuffer(),
        owner.toBuffer(),
        Buffer.from("ticket"),
      ],
      program.programId
    );

    return attendeeRecord;
  };

  const event = getEvent(0);
  const attendeeRecord1 = getAttendeeRecord(event, eventAttendee1.publicKey);
  const attendeeRecord2 = getAttendeeRecord(event, eventAttendee2.publicKey);
  const attendeeRecordRejected = getAttendeeRecord(
    event,
    eventAttendeeRejected.publicKey
  );
  // console.log("Provider pubkey", program.provider.publicKey.toString());

  it("creates community", async () => {
    await program.methods.createCommunity(seed, "testCommunity").rpc();

    const communityData = await program.account.community.fetch(community);
    assert.strictEqual(communityData.name, "testCommunity");
    assert.strictEqual(communityData.seed.toString(), seed.toString());
  });

  it("creates event", async () => {
    const timeNow = Date.now() / 1000;
    const tx = await program.methods
      .createEvent(
        "testEvent",
        "https://example.com/nft.json",
        { inPerson: {} },
        "testOrganizer",
        new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL),
        // event_starts_at
        new anchor.BN(timeNow + 2),
        // event ends_at
        new anchor.BN(timeNow + 100),
        // registration starts_at
        new anchor.BN(timeNow + 1),
        // registration ends at
        new anchor.BN(timeNow + 100),
        // capacity
        new anchor.BN(3),
        // location
        "testLocation",
        // virtual_link
        null,
        // description
        "testDescription",
        new anchor.BN(0),
        [eventAuthority.publicKey],
        true
      )
      .accountsPartial({
        community,
        authority: program.provider.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        mplCoreProgram: new anchor.web3.PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        ),
        rewardAccount: null,
        rewardMint: null,
        senderAccount: null,
      })
      .rpc();
    const eventData = await program.account.event.fetch(event);
    assert.strictEqual(
      eventData.commitmentFee.toNumber(),
      0.1 * anchor.web3.LAMPORTS_PER_SOL
    );
    assert.strictEqual(eventData.authorityMustSign, true);
    const eventCollection = getEventCollection(event);
    const umi = await createUmi();
    const eventCollectionData = await fetchCollectionV1(
      umi,
      publicKey(eventCollection)
    );
    assert.strictEqual(eventCollectionData.numMinted, 0);
    assert.strictEqual(eventCollectionData.name, "testEvent");
  });

  it("joins event - 3 attendees", async () => {
    // sleep for 2 seconds due to event not starting
    await sleep(2_000);
    await program.provider.connection.confirmTransaction(
      await program.provider.connection.requestAirdrop(
        eventAttendee1.publicKey,
        1 * anchor.web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );
    await program.provider.connection.confirmTransaction(
      await program.provider.connection.requestAirdrop(
        eventAuthority.publicKey,
        10 * anchor.web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );
    const joinEventIxn = await program.methods
      .joinEvent()
      .accountsPartial({
        community,
        event,
        eventAuthority: eventAuthority.publicKey,
        attendee: eventAttendee1.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        ),
      })
      .instruction();

    // include the eventAuthority as a signer
    joinEventIxn.keys = joinEventIxn.keys.map((key) => {
      if (key.pubkey.equals(eventAuthority.publicKey)) {
        return {
          pubkey: key.pubkey,
          isSigner: true,
          isWritable: false,
        };
      }
      return key;
    });

    const messageV0 = new anchor.web3.TransactionMessage({
      instructions: [joinEventIxn],
      payerKey: eventAttendee1.publicKey,
      recentBlockhash: (await program.provider.connection.getLatestBlockhash())
        .blockhash,
    }).compileToV0Message([]);

    const txJoinEvent = new anchor.web3.VersionedTransaction(messageV0);
    txJoinEvent.sign([eventAttendee1, eventAuthority]);

    await program.provider.connection.confirmTransaction(
      await program.provider.connection.sendRawTransaction(
        txJoinEvent.serialize()
      ),
      "confirmed"
    );

    await program.provider.connection.confirmTransaction(
      await program.provider.connection.requestAirdrop(
        eventAttendee2.publicKey,
        1 * anchor.web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );

    const joinEventIxn2 = await program.methods
      .joinEvent()
      .accountsPartial({
        community,
        event,
        eventAuthority: eventAuthority.publicKey,
        attendee: eventAttendee2.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        ),
      })
      .instruction();

    // include the eventAuthority as a signer
    joinEventIxn2.keys = joinEventIxn2.keys.map((key) => {
      if (key.pubkey.equals(eventAuthority.publicKey)) {
        return {
          pubkey: key.pubkey,
          isSigner: true,
          isWritable: false,
        };
      }
      return key;
    });

    const messageV02 = new anchor.web3.TransactionMessage({
      instructions: [joinEventIxn2],
      payerKey: eventAttendee2.publicKey,
      recentBlockhash: (await program.provider.connection.getLatestBlockhash())
        .blockhash,
    }).compileToV0Message([]);

    const txJoinEvent2 = new anchor.web3.VersionedTransaction(messageV02);
    txJoinEvent2.sign([eventAttendee2, eventAuthority]);

    await program.provider.connection.confirmTransaction(
      await program.provider.connection.sendRawTransaction(
        txJoinEvent2.serialize()
      ),
      "confirmed"
    );

    await program.provider.connection.confirmTransaction(
      await program.provider.connection.requestAirdrop(
        eventAttendeeRejected.publicKey,
        1 * anchor.web3.LAMPORTS_PER_SOL
      ),
      "confirmed"
    );
    const joinEventIxnRejected = await program.methods
      .joinEvent()
      .accountsPartial({
        community,
        event,
        eventAuthority: eventAuthority.publicKey,
        attendee: eventAttendeeRejected.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        ),
      })
      .instruction();

    // include the eventAuthority as a signer
    joinEventIxnRejected.keys = joinEventIxnRejected.keys.map((key) => {
      if (key.pubkey.equals(eventAuthority.publicKey)) {
        return {
          pubkey: key.pubkey,
          isSigner: true,
          isWritable: false,
        };
      }
      return key;
    });

    const messageV0Rejected = new anchor.web3.TransactionMessage({
      instructions: [joinEventIxnRejected],
      payerKey: eventAttendeeRejected.publicKey,
      recentBlockhash: (await program.provider.connection.getLatestBlockhash())
        .blockhash,
    }).compileToV0Message([]);

    const txJoinEventRejected = new anchor.web3.VersionedTransaction(
      messageV0Rejected
    );
    txJoinEventRejected.sign([eventAttendeeRejected, eventAuthority]);

    await program.provider.connection.confirmTransaction(
      await program.provider.connection.sendRawTransaction(
        txJoinEventRejected.serialize()
      ),
      "confirmed"
    );

    const umi = await createUmi();
    const eventCollection = getEventCollection(event);
    const eventCollectionData = await fetchCollectionV1(
      umi,
      publicKey(eventCollection)
    );
    assert.strictEqual(eventCollectionData.numMinted, 3);
    assert.strictEqual(eventCollectionData.currentSize, 3);
    const attendeeAsset = getEventTicketAsset(event, eventAttendee1.publicKey);
    const eventTicketAssetData = await fetchAssetV1(
      umi,
      publicKey(attendeeAsset)
    );
    assert.strictEqual(
      eventTicketAssetData.owner.toString(),
      eventAttendee1.publicKey.toString()
    );
    assert.strictEqual(
      eventTicketAssetData.name.toString(),
      "testEvent #" + "1"
    );

    assert.strictEqual(eventTicketAssetData.updateAuthority.type, "Collection");
    assert.strictEqual(
      eventTicketAssetData.updateAuthority.address.toString(),
      eventCollection.toString()
    );

    const attendeeDataRejected = await program.account.attendee.fetch(
      attendeeRecordRejected
    );
    assert.strictEqual(
      attendeeDataRejected.owner.toString(),
      eventAttendeeRejected.publicKey.toString()
    );
    const attendeeData = await program.account.attendee.fetch(attendeeRecord1);
    assert.strictEqual(
      attendeeData.owner.toString(),
      eventAttendee1.publicKey.toString()
    );
    assert.deepStrictEqual(attendeeData.status, { pending: {} });
  });

  it("joined attendee cannot rejoin", async () => {
    const rejoinJoinedEventIxn = await program.methods
      .joinEvent()
      .accountsPartial({
        community,
        event,
        eventAuthority: eventAuthority.publicKey,
        attendee: eventAttendee1.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        ),
      })
      .instruction();

    // include the eventAuthority as a signer
    rejoinJoinedEventIxn.keys = rejoinJoinedEventIxn.keys.map((key) => {
      if (key.pubkey.equals(eventAuthority.publicKey)) {
        return {
          pubkey: key.pubkey,
          isSigner: true,
          isWritable: false,
        };
      }
      return key;
    });

    const messageV0Rejoin = new anchor.web3.TransactionMessage({
      instructions: [rejoinJoinedEventIxn],
      payerKey: eventAttendee1.publicKey,
      recentBlockhash: (await program.provider.connection.getLatestBlockhash())
        .blockhash,
    }).compileToV0Message([]);

    const txJoinEventRejoin = new anchor.web3.VersionedTransaction(messageV0Rejoin);
    txJoinEventRejoin.sign([eventAttendee1, eventAuthority]);
    try {
      await program.provider.connection.sendRawTransaction(
        txJoinEventRejoin.serialize()
      );
      assert.fail("Transaction should have failed");
    } catch (error) {
      const logs = error.logs;
      assert.ok(logs.some((log: string) => log.includes("already in use")));
      assert.ok(logs.some((log: string) => log.includes("custom program error: 0x0")));
      assert.ok(logs.some((log: string) => log.includes(getAttendeeRecord(event, eventAttendee1.publicKey).toString())));
    }
  });

  it("reject attendenace", async () => {
    const rejectAttendanceIxn = await program.methods
      .rejectAttendee()
      .accountsPartial({
        community,
        event,
        eventAuthority: eventAuthority.publicKey,
        owner: eventAttendeeRejected.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        ),
      })
      .instruction();

    const messageV0RejectAttendance = new anchor.web3.TransactionMessage({
      instructions: [rejectAttendanceIxn],
      payerKey: eventAuthority.publicKey,
      recentBlockhash: (await program.provider.connection.getLatestBlockhash())
        .blockhash,
    }).compileToV0Message([]);

    const txRejectAttendance = new anchor.web3.VersionedTransaction(
      messageV0RejectAttendance
    );
    // must manually sign the transaction
    txRejectAttendance.sign([eventAuthority]);

    await program.provider.connection.confirmTransaction(
      await program.provider.connection.sendRawTransaction(
        txRejectAttendance.serialize()
      ),
      "confirmed"
    );
    const attendeeDataRejectedData = await program.account.attendee.fetch(
      attendeeRecordRejected
    );
    assert.strictEqual(
      attendeeDataRejectedData.owner.toString(),
      eventAttendeeRejected.publicKey.toString()
    );
    assert.deepStrictEqual(attendeeDataRejectedData.status, { rejected: {} });
  });

  it("rejected attendenace cannot verify attendance", async () => {
    const verifyAttendanceForRejectedIxn = await program.methods
      .verifyAttendee()
      .accountsPartial({
        community,
        event,
        eventAuthority: eventAuthority.publicKey,
        owner: eventAttendeeRejected.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        ),
      })
      .instruction();

    const messageV0VerifyAttendanceForRejected =
      new anchor.web3.TransactionMessage({
        instructions: [verifyAttendanceForRejectedIxn],
        payerKey: eventAuthority.publicKey,
        recentBlockhash: (
          await program.provider.connection.getLatestBlockhash()
        ).blockhash,
      }).compileToV0Message([]);

    const txVerifyAttendanceForRejected = new anchor.web3.VersionedTransaction(
      messageV0VerifyAttendanceForRejected
    );

    // must manually sign the transaction
    txVerifyAttendanceForRejected.sign([eventAuthority]);
    try {
      await program.provider.connection.sendRawTransaction(
        txVerifyAttendanceForRejected.serialize()
      );
      assert.fail("Transaction should have failed");
    } catch (error) {
      const logs = error.logs;
      assert.ok(logs.some((log: string) => log.includes("Error Code: AlreadyScanned")));
      assert.ok(logs.some((log: string) => log.includes("Error Number: 6014")));
      assert.ok(
        logs.some((log: string) => log.includes("Ticket has been signed already"))
      );
    }
  });

  it("verify attendenace", async () => {
    const verifyAttendanceIxn = await program.methods
      .verifyAttendee()
      .accountsPartial({
        community,
        event,
        eventAuthority: eventAuthority.publicKey,
        owner: eventAttendee1.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        ),
      })
      .instruction();

    const messageV0VerifyAttendance = new anchor.web3.TransactionMessage({
      instructions: [verifyAttendanceIxn],
      payerKey: eventAuthority.publicKey,
      recentBlockhash: (await program.provider.connection.getLatestBlockhash())
        .blockhash,
    }).compileToV0Message([]);

    const txVerifyAttendance = new anchor.web3.VersionedTransaction(
      messageV0VerifyAttendance
    );
    // must manually sign the transaction
    txVerifyAttendance.sign([eventAuthority]);

    await program.provider.connection.confirmTransaction(
      await program.provider.connection.sendRawTransaction(
        txVerifyAttendance.serialize()
      ),
      "confirmed"
    );

    const eventAttendee1Data = await program.account.attendee.fetch(
      attendeeRecord1
    );
    assert.strictEqual(
      eventAttendee1Data.owner.toString(),
      eventAttendee1.publicKey.toString()
    );
    assert.deepStrictEqual(eventAttendee1Data.status, { verified: {} });

    const umi = await createUmi();
    const attendeeAsset = getEventTicketAsset(event, eventAttendee1.publicKey);
    const eventTicketAssetData = await fetchAssetV1(
      umi,
      publicKey(attendeeAsset)
    );
    assert.strictEqual(
      eventTicketAssetData.owner.toString(),
      eventAttendee1.publicKey.toString()
    );
    assert.strictEqual(
      eventTicketAssetData.name.toString(),
      "testEvent #" + "1"
    );
    const verifiedBuffer = Uint8Array.from(Buffer.from("Verified"));
    assert.strictEqual(
      eventTicketAssetData.appDatas?.[0].data.toString(),
      verifiedBuffer.toString()
    );
  });

  it("verified attendenace cannot reverify", async () => {
    const verifyAttendanceForVerifiedIxn = await program.methods
      .verifyAttendee()
      .accountsPartial({
        community,
        event,
        eventAuthority: eventAuthority.publicKey,
        owner: eventAttendee1.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey(
          "CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"
        ),
      })
      .instruction();

    const messageV0VerifyAttendanceForVerified =
      new anchor.web3.TransactionMessage({
        instructions: [verifyAttendanceForVerifiedIxn],
        payerKey: eventAuthority.publicKey,
        recentBlockhash: (
          await program.provider.connection.getLatestBlockhash()
        ).blockhash,
      }).compileToV0Message([]);

    const txVerifyAttendanceForVerified = new anchor.web3.VersionedTransaction(
      messageV0VerifyAttendanceForVerified
    );

    // must manually sign the transaction
    txVerifyAttendanceForVerified.sign([eventAuthority]);
    try {
      await program.provider.connection.sendRawTransaction(
        txVerifyAttendanceForVerified.serialize()
      );
      assert.fail("Transaction should have failed");
    } catch (error) {
      const logs = error.logs;
      assert.ok(logs.some((log: string) => log.includes("Error Code: AlreadyScanned")));
      assert.ok(logs.some((log: string) => log.includes("Error Number: 6014")));
      assert.ok(
        logs.some((log: string) => log.includes("Ticket has been signed already"))
      );
    }
  });

  it("claim rewards", async () => {
    await program.methods
      .claimRewards()
      .accountsPartial({
        community,
        event,
        claimer: eventAttendee1.publicKey,
        attendeeRecord: getAttendeeRecord(event, eventAttendee1.publicKey),
        rewardAccount: null,
        receiverAccount: null,
        rewardMint: null,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([eventAttendee1])
      .rpc();
    const eventAttendee1Data = await program.account.attendee.fetch(
      attendeeRecord1
    );
    assert.deepStrictEqual(eventAttendee1Data.status, { claimed: {} });
  });

  it("claimed rewards cannot be reclaimed", async () => {
    const reclaimRewardIxn = await program.methods
      .claimRewards()
      .accountsPartial({
        community,
        event,
        claimer: eventAttendee1.publicKey,
        attendeeRecord: getAttendeeRecord(event, eventAttendee1.publicKey),
        rewardAccount: null,
        receiverAccount: null,
        rewardMint: null,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .instruction();
      
    const messageV0ReclaimReward =
    new anchor.web3.TransactionMessage({
      instructions: [reclaimRewardIxn],
      payerKey: eventAttendee1.publicKey,
      recentBlockhash: (
        await program.provider.connection.getLatestBlockhash()
      ).blockhash,
    }).compileToV0Message([]);

  const txReclaimReward = new anchor.web3.VersionedTransaction(
    messageV0ReclaimReward
  );

  // must manually sign the transaction
  txReclaimReward.sign([eventAttendee1]);
    try {
      await program.provider.connection.sendRawTransaction(
        txReclaimReward.serialize()
      );
      assert.fail("Transaction should have failed");
    } catch (error) {
      const logs = error.logs;
      assert.ok(logs.some((log: string) => log.includes("Error Code: AlreadyClaimed")));
      assert.ok(logs.some((log: string) => log.includes("Error Number: 6009")));
      assert.ok(
        logs.some((log: string) => log.includes("this attendee has already claimed the rewards."))
      );
    }
  });

  it("claim rewards fail for unattended claimer", async () => {
    const claimRewardIxn = await program.methods
      .claimRewards()
      .accountsPartial({
        community,
        event,
        claimer: eventAttendee2.publicKey,
        attendeeRecord: getAttendeeRecord(event, eventAttendee2.publicKey),
        rewardAccount: null,
        receiverAccount: null,
        rewardMint: null,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .instruction();
      
    const messageV0ClaimReward =
    new anchor.web3.TransactionMessage({
      instructions: [claimRewardIxn],
      payerKey: eventAttendee2.publicKey,
      recentBlockhash: (
        await program.provider.connection.getLatestBlockhash()
      ).blockhash,
    }).compileToV0Message([]);

  const txClaimReward = new anchor.web3.VersionedTransaction(
    messageV0ClaimReward
  );

  // must manually sign the transaction
  txClaimReward.sign([eventAttendee2]);
    try {
      await program.provider.connection.sendRawTransaction(
        txClaimReward.serialize()
      );
      assert.fail("Transaction should have failed");
    } catch (error) {
      const logs = error.logs;
      assert.ok(logs.some((log: string) => log.includes("Error Code: AttendeeStatusPending")));
      assert.ok(logs.some((log: string) => log.includes("Error Number: 6006")));
      assert.ok(
        logs.some((log: string) => log.includes("The rewards cannot be claimed during the pending status."))
      );
    }
  });

  it("claim rewards of rejected attendee by community authority", async () => {
    await program.methods
      .claimRewards()
      .accountsPartial({
        community,
        event,
        claimer: program.provider.publicKey,
        attendeeRecord: getAttendeeRecord(
          event,
          eventAttendeeRejected.publicKey
        ),
        rewardAccount: null,
        receiverAccount: null,
        rewardMint: null,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .rpc();
    const attendeeDataRejected = await program.account.attendee.fetch(
      attendeeRecordRejected
    );
    assert.deepStrictEqual(attendeeDataRejected.status, { claimed: {} });
  });
});
