import * as anchor from "@coral-xyz/anchor";
import { chaiSolana } from "@saberhq/chai-solana";
import { SolanaProvider } from "@saberhq/solana-contrib";
import chai from "chai";

import { MockOracles } from "../src";

chai.use(chaiSolana);

export const makeSDK = (): MockOracles => {
  const anchorProvider = anchor.AnchorProvider.env();
  anchor.setProvider(anchorProvider);

  const provider = SolanaProvider.init({
    connection: anchorProvider.connection,
    wallet: anchorProvider.wallet,
    opts: anchorProvider.opts,
  });

  return MockOracles.load({
    provider,
  });
};
