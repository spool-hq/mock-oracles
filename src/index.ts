import { Program } from "@coral-xyz/anchor";
import {
  Keypair,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  Signer,
  PublicKey,
} from "@solana/web3.js";
import BN from "bn.js";
import {
  AugmentedProvider,
  Provider,
  SolanaAugmentedProvider,
} from "@saberhq/solana-contrib";

import { MockOraclesIDL, MockOraclesJSON } from "./idls/mock_oracles";
import { newProgram } from "@saberhq/anchor-contrib";
import { MOCK_ORACLES_ADDRESS } from "./constants";

enum MockOracleAccountType {
  PYTH_PRICE,
  PYTH_PRODUCT,
  SWITCHBOARD,
}

export interface PythPriceParams {
  price?: BN;
  conf?: BN;
  expo?: number;
  ema_price?: BN;
  ema_conf?: BN;
}

export interface SwitchboardPriceParams {
  price?: BN;
  expo?: number;
}

export class MockOracles {
  public readonly PYTH_PRICE_ACCOUNT_SIZE = 3312;
  public readonly PYTH_PRODUCT_ACCOUNT_SIZE = 512;
  public readonly SWITCHBOARD_ACCOUNT_SIZE = 3851;

  constructor(
    readonly provider: AugmentedProvider,
    readonly program: Program<MockOraclesIDL>
  ) {}

  /**
   * Creates a new instance of the SDK with the given keypair.
   */
  withSigner(signer: Signer): MockOracles {
    return MockOracles.load({
      provider: this.provider.withSigner(signer),
      programId: this.program.programId,
    });
  }

  /**
   * Loads the SDK.
   */
  static load({
    provider,
    programId,
  }: {
    // Provider
    provider: Provider;
    programId?: PublicKey;
  }): MockOracles {
    const program = newProgram<Program<MockOraclesIDL>>(
      MockOraclesJSON,
      programId ?? MOCK_ORACLES_ADDRESS,
      provider
    );
    return new MockOracles(new SolanaAugmentedProvider(provider), program);
  }

  async createPyth(): Promise<{
    priceKeypair: Keypair;
    productKeypair: Keypair;
  }> {
    const [priceKeypair, createPriceIx] = await this._createAccount(
      MockOracleAccountType.PYTH_PRICE
    );
    const [productKeypair, createProductIx] = await this._createAccount(
      MockOracleAccountType.PYTH_PRODUCT
    );
    const initPythTx = await this.program.methods
      .initPyth()
      .accounts({
        priceAccount: priceKeypair.publicKey,
        productAccount: productKeypair.publicKey,
      })
      .preInstructions([createPriceIx, createProductIx])
      .transaction();

    const pendingTx = await this.provider.send(
      initPythTx,
      [priceKeypair, productKeypair],
      {
        commitment: "confirmed",
        skipPreflight: true,
      }
    );
    await pendingTx.wait();

    return { priceKeypair, productKeypair };
  }

  async createSwitchboard(): Promise<{
    switchboardKeypair: Keypair;
  }> {
    const [switchboardKeypair, createSwitchboardIx] = await this._createAccount(
      MockOracleAccountType.SWITCHBOARD
    );
    const initSwitchboardIx = await this.program.methods
      .initSwitchboard()
      .accounts({
        target: switchboardKeypair.publicKey,
      })
      .instruction();

    const createTx = new Transaction().add(
      createSwitchboardIx,
      initSwitchboardIx
    );
    const pendingTx = await this.provider.send(createTx, [switchboardKeypair], {
      commitment: "confirmed",
      skipPreflight: true,
    });
    await pendingTx.wait();

    return { switchboardKeypair };
  }

  async setPythPrice(
    keypair: Keypair,
    {
      price = new BN(0),
      conf = new BN(0),
      expo = 0,
      ema_price = new BN(0),
      ema_conf = new BN(0),
    }: PythPriceParams
  ): Promise<string> {
    const tx = await this.program.methods
      .setPythPrice(price, conf, expo, ema_price, ema_conf)
      .accounts({
        target: keypair.publicKey,
      })
      .signers([keypair])
      .transaction();
    const pendingTx = await this.provider.send(tx);
    return (await pendingTx.wait()).signature;
  }

  async setSwitchboardPrice(
    keypair: Keypair,
    { price = new BN(-1), expo = 0 }: SwitchboardPriceParams
  ): Promise<string> {
    const tx = await this.program.methods
      .setSwitchboardPrice(price, expo)
      .accounts({
        target: keypair.publicKey,
      })
      .signers([keypair])
      .transaction();
    const pendingTx = await this.provider.send(tx, [], { skipPreflight: true });
    return (await pendingTx.wait()).signature;
  }

  private _space(type: MockOracleAccountType): number {
    switch (type) {
      case MockOracleAccountType.PYTH_PRICE:
        return this.PYTH_PRICE_ACCOUNT_SIZE;
      case MockOracleAccountType.PYTH_PRODUCT:
        return this.PYTH_PRODUCT_ACCOUNT_SIZE;
      case MockOracleAccountType.SWITCHBOARD:
        return this.SWITCHBOARD_ACCOUNT_SIZE;
    }
  }

  private async _createAccount(
    type: MockOracleAccountType
  ): Promise<[Keypair, TransactionInstruction]> {
    const newAccount = Keypair.generate();
    const space = this._space(type);
    const neededBalance =
      await this.provider.connection.getMinimumBalanceForRentExemption(space);

    const ix = SystemProgram.createAccount({
      fromPubkey: this.provider.wallet.publicKey,
      newAccountPubkey: newAccount.publicKey,
      programId: this.program.programId,
      lamports: neededBalance,
      space,
    });

    return [newAccount, ix];
  }
}
