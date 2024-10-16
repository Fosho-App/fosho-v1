import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { FoshoProgram } from "../target/types/fosho_program";
import * as token from "@solana/spl-token";
import pvt from "../../../../sol/id.json";

describe("fosho-program", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.FoshoProgram as Program<FoshoProgram>;
  const keypair = anchor.web3.Keypair.fromSecretKey(Uint8Array.from(pvt));

  const seed = anchor.web3.Keypair.generate().publicKey;

  const [community] = anchor.web3.PublicKey.findProgramAddressSync([
    Buffer.from("community"),
    seed.toBuffer()
  ], 
    program.programId
  );

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

  xit("creates community", async () => {
    const tx = await program.methods.createCommunity(seed).rpc();
    console.log("Your transaction signature", tx);
    console.log("seed of the community: ", seed);
  });

  it("creates event with 5 attendees and no reward", async () => {
    const nonce = 0
    const event = getEvent(0)

    const tx = await program.methods.createEvent(
      0,
      2,
      new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL),
      new anchor.BN(Date.now()/1000 + 86400),
      null,
      new anchor.BN(0),
    ).accountsPartial({
      community,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      rewardAccount: null,
      rewardMint: null,
      senderAccount: null
    })
    .rpc();
    console.log("Your transaction signature", tx);

    const eventData = await program.account.event.fetch(event)
    console.log(eventData)
  })

  it("creates event with 2 attendees and 400 TOKEN reward each", async () => {
    const nonce = 1;
    const event = getEvent(nonce)
    const rewardAccount = anchor.utils.token.associatedAddress({mint, owner: event});

    const tx = await program.methods.createEvent(
      nonce,
      5,
      new anchor.BN(0.1 * anchor.web3.LAMPORTS_PER_SOL),
      new anchor.BN(Date.now()/1000 + 86400),
      null,
      new anchor.BN(400),
    ).accountsPartial({
      community,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      rewardAccount,
      rewardMint: mint,
      senderAccount: ata
    })
    .rpc();
    console.log("Your transaction signature", tx);

    const eventData = await program.account.event.fetch(event)
    console.log(eventData)
  })
});
