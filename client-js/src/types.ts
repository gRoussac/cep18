import { type BigNumberish } from '@ethersproject/bignumber';
import { CLPublicKey } from 'casper-js-sdk';

export enum EVENTS_MODE {
  NoEvents = 0,
  CES = 1,
  Native = 2,
  NativeNCES = 3
}

export interface InstallArgs {
  /** token name */
  name: string;
  /** token symbol */
  symbol: string;
  /** token decimals */
  decimals: BigNumberish;
  /** token total supply */
  totalSupply: BigNumberish;
  /** events mode, disabled by default */
  eventsMode?: EVENTS_MODE;
  /** flag for mint and burn, false by default */
  enableMintAndBurn?: boolean;
}

export interface TransferableArgs {
  amount: BigNumberish;
}

export interface TransferArgs extends TransferableArgs {
  recipient: CLPublicKey;
}

export interface TransferFromArgs extends TransferArgs {
  owner: CLPublicKey;
}

export interface ApproveArgs extends TransferableArgs {
  spender: CLPublicKey;
}

export interface MintArgs extends TransferableArgs {
  owner: CLPublicKey;
}

export interface BurnArgs extends TransferableArgs {
  owner: CLPublicKey;
}

export interface ChangeSecurityArgs {
  adminList?: CLPublicKey[];
  minterList?: CLPublicKey[];
  burnerList?: CLPublicKey[];
  mintAndBurnList?: CLPublicKey[];
  noneList?: CLPublicKey[];
}
