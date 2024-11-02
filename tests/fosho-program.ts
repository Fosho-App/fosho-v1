import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import {FoshoProgram} from "../target/types/fosho_program";
import crypto from 'crypto';

const sleep = (ms: number) => require('timers/promises').setTimeout(ms);

export function createKnownTestKeypair(knownKey: string) {
  try {
    const deterministicSalt = crypto
      .createHash('sha256')
      .update(knownKey)
      .digest();

    const newKey = anchor.web3.Keypair.fromSeed(new Uint8Array(deterministicSalt));
    console.log(knownKey, 'Public Key:', newKey.publicKey.toString());
    return newKey;
  } catch (error) {
    throw new Error(`Failed to create keypair: ${error}`);
  }
}


describe("fosho-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.FoshoProgram as Program<FoshoProgram>;
  
  const seed = anchor.web3.Keypair.generate().publicKey;

  const [community] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from("community"),
    seed.toBuffer()
  ], 
    program.programId
  );

  console.log("Community pubkey", community.toString())

  const eventAuthority = createKnownTestKeypair("eventAuthority");
  const eventAttendee1 = createKnownTestKeypair("eventAttendee1");
  const eventCollection = anchor.web3.Keypair.generate();
  const eventAsset1 = anchor.web3.Keypair.generate();
  console.log("eventCollection pubkey", eventCollection.publicKey.toString())
  console.log("eventAsset1 pubkey", eventAsset1.publicKey.toString())


  let mint = new anchor.web3.PublicKey("Db9vq4fNEtaTGUATzibXY2NC7hvFTVPH2w6psYBFyUKn");
  let ata = new anchor.web3.PublicKey("4cf8Yu7pLDEGzstmQV6Two476dTvb7hC5u8ZtjwcuzX1");

  const getEvent = (nonce: number) => {
    const [event] = anchor.web3.PublicKey.findProgramAddressSync([
      Buffer.from("event"),
      community.toBuffer(),
      new anchor.BN(nonce).toArrayLike(Buffer, 'le', 4)
    ], 
      program.programId
    );

    return event
  }

  const getAttendeeRecord = (event: anchor.web3.PublicKey, 
    eventAttendee: anchor.web3.PublicKey) => {
    const [attendeeRecord] = anchor.web3.PublicKey.findProgramAddressSync([
      Buffer.from("attendee"),
      event.toBuffer(),
      eventAttendee.toBuffer(),
    ], 
      program.programId
    );

    return attendeeRecord
  }


  it("creates community", async () => {
    const tx = await program.methods.createCommunity(seed, "testCommunity").rpc();
    console.log("Your transaction signature", tx);
    console.log("Community Seed: ", seed.toString());
  });

  it("creates event, joins event, verify attendance", async () => {
      const event = getEvent(0)
      const attendeeRecord = getAttendeeRecord(event, eventAttendee1.publicKey)
      console.log("Provider pubkey", program.provider.publicKey.toString())
      const timeNow = Date.now() / 1000;
      const tx = await program.methods.createEvent(
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
        new anchor.BN(timeNow + 10),
        // capacity
        new anchor.BN(10),
        // location
        "testLocation",
        // virtual_link
        null,
        // description
        "testDescription",
        new anchor.BN(0),
        [eventAuthority.publicKey],
        true,
      ).accountsPartial({
        community,
        authority: program.provider.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        eventCollection: eventCollection.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"),
        rewardAccount: null,
        rewardMint: null,
        senderAccount: null
      }).signers([eventCollection])
      .rpc();
      console.log("Your transaction signature", tx);
  
      const eventData = await program.account.event.fetch(event)
      console.log(eventData)

      console.log("attendeeRecord", attendeeRecord.toString())
      // sleep for 2 seconds
      await sleep(2_000)
      await program.provider.connection.confirmTransaction(
        await program.provider.connection.requestAirdrop(
          eventAttendee1.publicKey,
          10 * anchor.web3.LAMPORTS_PER_SOL
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
      const joinEventIxn = await program.methods.joinEvent().accountsPartial({
        community,
        event,
        eventAuthority: eventAuthority.publicKey,
        ticket: eventAsset1.publicKey,
        attendee: eventAttendee1.publicKey,
        eventCollection: eventCollection.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"),
      })
      .instruction();

      joinEventIxn.keys = joinEventIxn.keys.map((key) => {
        if (key.pubkey.equals(eventAuthority.publicKey)) {
          return {
            pubkey: key.pubkey,
            isSigner: true,
            isWritable: false,
          };
        }
        return key;
      })

      const messageV0 = new anchor.web3.TransactionMessage({
        instructions: [joinEventIxn],
        payerKey: eventAttendee1.publicKey,
        recentBlockhash: (await program.provider.connection.getLatestBlockhash()).blockhash
      }).compileToV0Message([]);

      const txJoinEvent = new anchor.web3.VersionedTransaction(messageV0);
      txJoinEvent.sign([eventAsset1, eventAttendee1, eventAuthority]);

      await program.provider.connection.confirmTransaction(
        await program.provider.connection.sendRawTransaction(txJoinEvent.serialize()),
        "confirmed"
      );
      console.log("txn sent")
      const attendeeData = await program.account.attendee.fetch(attendeeRecord)
      console.log(attendeeData)


      const verifyAttendanceIxn = await program.methods.verifyAttendee().accountsPartial({
        community,
        event,
        eventAuthority: eventAuthority.publicKey,
        ticket: eventAsset1.publicKey,
        owner: eventAttendee1.publicKey,
        eventCollection: eventCollection.publicKey,
        mplCoreProgram: new anchor.web3.PublicKey("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d"),
      })
      .instruction();

      const messageV0VerifyAttendance = new anchor.web3.TransactionMessage({
        instructions: [verifyAttendanceIxn],
        payerKey: eventAuthority.publicKey,
        recentBlockhash: (await program.provider.connection.getLatestBlockhash()).blockhash
      }).compileToV0Message([]);

      const txVerifyAttendance = new anchor.web3.VersionedTransaction(messageV0VerifyAttendance);
      txVerifyAttendance.sign([eventAuthority]);

      await program.provider.connection.confirmTransaction(
        await program.provider.connection.sendRawTransaction(txVerifyAttendance.serialize()),
        "confirmed"
      );
      console.log("txn sent")
      const attendeeData2 = await program.account.attendee.fetch(attendeeRecord)
      console.log(attendeeData2)

      

  })
  // it("creates event with 5 attendees and no reward", async () => {
  //   const nonce = 0
  //   const event = getEvent(0)

  //   const tx = await program.methods.createEvent(
  //     2,
  //     new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL),
  //     new anchor.BN(Date.now()/1000 + 86400),
  //     null,
  //     new anchor.BN(0),
  //   ).accountsPartial({
  //     community,
  //     tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
  //     rewardAccount: null,
  //     rewardMint: null,
  //     senderAccount: null
  //   })
  //   .rpc();
  //   console.log("Your transaction signature", tx);

  //   const eventData = await program.account.event.fetch(event)
  //   console.log(eventData)
  // })

  // it("creates event with 2 attendees and 400 TOKEN reward each", async () => {
  //   const nonce = 1;
  //   const event = getEvent(nonce)
  //   const rewardAccount = anchor.utils.token.associatedAddress({mint, owner: event});

  //   const tx = await program.methods.createEvent(
  //     5,
  //     new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL),
  //     new anchor.BN(Date.now()/1000 + 86400),
  //     null,
  //     new anchor.BN(400),
  //   ).accountsPartial({
  //     community,
  //     tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
  //     rewardAccount,
  //     rewardMint: mint,
  //     senderAccount: ata
  //   })
  //   .rpc();
  //   console.log("Your transaction signature", tx);

  //   const eventData = await program.account.event.fetch(event)
  //   console.log(eventData)
  // })
});
