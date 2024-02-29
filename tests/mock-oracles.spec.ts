import * as anchor from "@coral-xyz/anchor";
import { AggregatorAccountData } from "@switchboard-xyz/solana.js/generated";
import { Connection, PublicKey } from "@solana/web3.js";
import { parsePriceData } from "@pythnetwork/client";
import { assert } from "chai";

import { makeSDK } from "./workspace";

async function loadZeroCopyAggregator(
  con: Connection,
  pubkey: PublicKey
): Promise<AggregatorAccountData> {
  const accountInfo = await con.getAccountInfo(pubkey);
  const buffer = accountInfo?.data ?? Buffer.from("");
  return AggregatorAccountData.decode(buffer);
}

describe("Test Mock Oracles", () => {
  const mockOracles = makeSDK();
  const provider = mockOracles.provider;

  it("Write Pyth Data", async () => {
    const { priceKeypair } = await mockOracles.createPyth();
    const price = 10;

    await mockOracles.setPythPrice(priceKeypair, {
      price: new anchor.BN(price),
    });
    let pythData = await provider.connection.getAccountInfo(
      priceKeypair.publicKey
    );
    let pythPriceRecord = parsePriceData(pythData.data);
    assert(pythPriceRecord.price === price);
    assert(pythPriceRecord.exponent === 0);

    await mockOracles.setPythPrice(priceKeypair, {
      price: new anchor.BN(price * 2),
    });
    pythData = await provider.connection.getAccountInfo(priceKeypair.publicKey);
    pythPriceRecord = parsePriceData(pythData.data);
    assert(pythPriceRecord.price === price * 2);
  });

  it("Write Switchboard Data", async () => {
    const { switchboardKeypair } = await mockOracles.createSwitchboard();
    const price = 10;

    await mockOracles.setSwitchboardPrice(switchboardKeypair, {
      price: new anchor.BN(price),
    });
    let switchboardPrice = await loadZeroCopyAggregator(
      provider.connection,
      switchboardKeypair.publicKey
    );
    assert(
      switchboardPrice.latestConfirmedRound.result.toString() ===
        price.toString()
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

    await mockOracles.setSwitchboardPrice(switchboardKeypair, {});
    switchboardPrice = await loadZeroCopyAggregator(
      provider.connection,
      switchboardKeypair.publicKey
    );
  });
});
