import * as anchor from "@coral-xyz/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import { parsePriceData } from "@pythnetwork/client";
import { assert } from "chai";
import { makeSDK } from "./workspace";
import { AggregatorAccountData } from "@switchboard-xyz/solana.js/generated";

async function loadZeroCopyAggregator(
  con: Connection,
  pubkey: PublicKey
): Promise<AggregatorAccountData> {
  const accountInfo = await con.getAccountInfo(pubkey);
  const buffer = accountInfo?.data ?? Buffer.from("");
  return AggregatorAccountData.decode(buffer);
}

describe("Test Mock Oracles", () => {
  const _mockOracles = makeSDK();
  const provider = _mockOracles.provider;
  const mockOracles = _mockOracles.withSigner(
    (provider.wallet as anchor.Wallet).payer
  );

  it("Write Pyth Data", async () => {
    const { priceKeypair } = await mockOracles.createPyth();
    const price = 10;
    const slot = 10;

    await mockOracles.setPythPrice(priceKeypair, {
      price: new anchor.BN(price),
      slot: new anchor.BN(slot),
    });
    let pythData = await provider.connection.getAccountInfo(
      priceKeypair.publicKey
    );
    let pythPriceRecord = parsePriceData(pythData.data);
    assert(pythPriceRecord.price === price);
    assert(pythPriceRecord.exponent === 0);
    assert(pythPriceRecord.validSlot.toString() === slot.toString());

    await mockOracles.setPythPrice(priceKeypair, {
      price: new anchor.BN(price * 2),
    });
    pythData = await provider.connection.getAccountInfo(priceKeypair.publicKey);
    pythPriceRecord = parsePriceData(pythData.data);
    assert(pythPriceRecord.price === price * 2);

    await mockOracles.setPythPrice(priceKeypair, {
      slot: new anchor.BN(slot * 2),
    });
    pythData = await provider.connection.getAccountInfo(priceKeypair.publicKey);
    pythPriceRecord = parsePriceData(pythData.data);
    assert(pythPriceRecord.validSlot.toString() === (slot * 2).toString());
  });

  it("Write Switchboard Data", async () => {
    const { switchboardKeypair } = await mockOracles.createSwitchboard();
    const price = 10;
    const slot = 10;

    await mockOracles.setSwitchboardPrice(switchboardKeypair, {
      price: new anchor.BN(price),
      slot: new anchor.BN(slot),
    });
    let switchboardPrice = await loadZeroCopyAggregator(
      provider.connection,
      switchboardKeypair.publicKey
    );
    assert(
      switchboardPrice.latestConfirmedRound.result.toString() ===
        price.toString()
    );
    assert(
      switchboardPrice.latestConfirmedRound.roundOpenSlot.toString() ===
        slot.toString()
    );

    await mockOracles.setSwitchboardPrice(switchboardKeypair, {
      price: new anchor.BN(price * 2),
    });
    switchboardPrice = await loadZeroCopyAggregator(
      provider.connection,
      switchboardKeypair.publicKey
    );
    assert(
      switchboardPrice.latestConfirmedRound.result.toString() ===
        (price * 2).toString()
    );

    await mockOracles.setSwitchboardPrice(switchboardKeypair, {
      slot: new anchor.BN(slot * 2),
    });
    switchboardPrice = await loadZeroCopyAggregator(
      provider.connection,
      switchboardKeypair.publicKey
    );
    assert(
      switchboardPrice.latestConfirmedRound.roundOpenSlot.toString() ===
        (slot * 2).toString()
    );
  });
});
