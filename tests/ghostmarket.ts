import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Ghostmarket } from "../target/types/ghostmarket";
import { assert } from "chai";

describe("ghostmarket", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.Ghostmarket as Program<Ghostmarket>;

  it("Farmer creates a listing", async () => {
    const farmer = provider.wallet;
    const productName = "Tomatoes";
    const deadline = Math.floor(Date.now() / 1000) + 3600;

    const [listingPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("listing"),
        farmer.publicKey.toBuffer(),
        Buffer.from(productName),
      ],
      program.programId
    );

    await program.methods
      .createListing(
        productName,
        new anchor.BN(100),
        new anchor.BN(1_000_000),
        new anchor.BN(deadline)
      )
      .accounts({
        listing: listingPda,
        farmer: farmer.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();

    const listing = await program.account.listing.fetch(listingPda);
    assert.equal(listing.productName, "Tomatoes");
    assert.equal(listing.isActive, true);
    console.log("✅ Listing created:", listing.productName);
  });
});