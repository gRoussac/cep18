/* eslint-disable @typescript-eslint/no-unsafe-return */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-call */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
import {
  AddressableEntityWrapper,
  CasperServiceByJsonRPC,
  type CLPublicKey,
  type GetDeployResult,
  NamedKey,
} from 'casper-js-sdk';

type Account = {
  addressableEntity: AddressableEntityWrapper;
  namedKeys: NamedKey[];
};

export const getAccountInfo = async (
  nodeAddress: string,
  publicKey: CLPublicKey
): Promise<Account> => {
  const client = new CasperServiceByJsonRPC(nodeAddress);
  // TODO GR
  // const { AddressableEntity } = await client.getEntity({
  //   AccountHash: publicKey.toAccountHash().toFormattedString()
  // });
  const addressableEntity = await client.getAccountInfo(publicKey.toAccountHash());
  if (!test) throw Error('Not found account');
  const namedKeys = addressableEntity.account?.named_keys;

  return {
    addressableEntity,
    namedKeys
  };
};

export const findKeyFromAccountNamedKeys = (
  account: Account,
  name: string
): string => {
  const key = account.namedKeys.find(namedKey => namedKey.name === name)?.key;

  if (!key) throw Error(`Not found key: ${name}`);

  return key;
};

export const sleep = async (ms: number): Promise<void> => {
  // eslint-disable-next-line no-promise-executor-return
  await new Promise(resolve => setTimeout(resolve, ms));
};

export const expectDeployResultToSuccess = (result: GetDeployResult): void => {
  if (
    result.execution_info &&
    result.execution_info.execution_result &&
    'Version2' in result.execution_info.execution_result
  ) {
    const v2 = result.execution_info.execution_result.Version2;

    expect(v2.error_message).toBeNull();
  } else {
    fail('Not found Version2 in execution_result');
  }
};