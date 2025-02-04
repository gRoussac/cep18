/* eslint-disable @typescript-eslint/no-use-before-define */
/* eslint-disable @typescript-eslint/no-unsafe-argument */
/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-call */
// eslint-disable-next-line eslint-comments/disable-enable-pair
/* eslint-disable no-console */
import { type BigNumberish } from '@ethersproject/bignumber';
import { BigNumber } from '@ethersproject/bignumber';
import {
  CasperServiceByJsonRPC,
  type CLPublicKey,
  EventStream
} from 'casper-js-sdk';
import { encodeBase16 } from 'casper-js-sdk';
import { AsymmetricKey } from 'casper-js-sdk/dist/lib/Keys';

import {
  CEP18Client,
  ChangeSecurityArgs,
  ContractWASM,
  EVENTS_MODE,
  InstallArgs
} from '../../src';
import {
  DEPLOY_TIMEOUT,
  EVENT_STREAM_ADDRESS,
  NETWORK_NAME,
  NODE_URL,
  users
} from '../config';
import { findKeyFromAccountNamedKeys, getAccountInfo, sleep } from '../utils';
import { Deploy } from 'casper-js-sdk/dist/lib/DeployUtil';

describe('CEP18Client', () => {
  let testNumber = 1;
  const initialSupply = 10000;
  const client = new CasperServiceByJsonRPC(NODE_URL, true);
  const eventStream = new EventStream(EVENT_STREAM_ADDRESS);
  const owner = users[0];
  const user1 = users[1];
  const user2 = users[2];
  const user3 = users[3];
  const eventsMode = EVENTS_MODE.CES;

  let mintAndBurnTestsCep18: CEP18Client;
  let ownerMintAndBurnTestsCep18: CEP18Client;
  // beforeAll(async () => {
  //   const results = await Promise.all([
  //     await prepareContract(),
  //     await prepareContract()
  //   ]);
  //   mintAndBurnTestsCep18 = results[0].cep18;
  //   ownerMintAndBurnTestsCep18 = results[1].cep18;
  // });

  const doApprove = async (
    cep18: CEP18Client,
    spenderPubkey: CLPublicKey,
    amount: BigNumberish
  ): Promise<void> => {
    const deploy = cep18.approve(
      {
        spender: spenderPubkey,
        amount
      },
      5_000_000_000,
      owner.publicKey,
      NETWORK_NAME,
      [owner]
    );

    await deploy.send(NODE_URL);
    const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
    if (!result || !client.isDeploySuccessfull(result)) {
      fail('Transfer deploy failed');
    }
    const execution_result = result.execution_info!.execution_result!;
    const events = cep18.parseExecutionResult(execution_result);
    expect(events.length).toEqual(1);
    expect(events[0].name).toEqual('SetAllowance');
    await sleep(5000);
    const allowances = await cep18.allowances(owner.publicKey, spenderPubkey);
    expect(allowances.toNumber()).toEqual(amount);
  };

  const prepareContract = async (): Promise<{
    cep18: CEP18Client;
    tokenInfo: InstallArgs;
  }> => {
    testNumber += 1;
    const ordinal = testNumber;
    const random_part = `${Date.now()}_${ordinal}`;
    const tokenInfo: InstallArgs = {
      name: `Test_Token_${random_part}`,
      symbol: `TFT_${random_part}`,
      decimals: 2,
      totalSupply: initialSupply
    };

    const cep18 = new CEP18Client(NODE_URL, NETWORK_NAME);
    cep18.setContractName(`cep18_contract_hash_${tokenInfo.name}`);
    const deploy: Deploy = cep18.install(
      ContractWASM,
      { ...tokenInfo, eventsMode, enableMintAndBurn: true },
      350_000_000_000,
      owner.publicKey,
      NETWORK_NAME,
      [owner]
    );

    // TODO GR Deprecated send
    console.warn = () => { };

    await deploy.send(NODE_URL);
    const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
    if (!result || !client.isDeploySuccessfull(result)) {
      fail('Install deploy failed');
    }
    // await sleep(5000);
    const accountInfo = await getAccountInfo(NODE_URL, owner.publicKey);
    const contractHash = findKeyFromAccountNamedKeys(
      accountInfo,
      `cep18_contract_hash_${tokenInfo.name}`
    ).replace('entity-contract-', 'hash-');
    cep18.setContractHash(contractHash);
    await cep18.setupEventStream(eventStream);
    console.log(cep18, tokenInfo);
    return { cep18, tokenInfo };
  };

  afterAll(() => {
    eventStream.stop();
  });

  describe('Fetching metadata', () => {
    let cep18: CEP18Client;
    let tokenInfo: InstallArgs;
    beforeAll(async () => {
      const contractPreparationResult = await prepareContract();
      cep18 = contractPreparationResult.cep18;
      tokenInfo = contractPreparationResult.tokenInfo;
    });

    it('should deploy contract', async () => {
      const contracHash = cep18.contractHash;
      expect(contracHash).toBeDefined();
    });

    it('should match on-chain info with install info', async () => {
      console.log(cep18.contractHash);
      const name = await cep18.name();
      // const symbol = await cep18.symbol();
      // const decimals = await cep18.decimals();
      // const totalSupply = await cep18.totalSupply();
      // const isMintAndBurnEnabled = await cep18.isMintAndBurnEnabled();

      // expect(name).toBe(tokenInfo.name);
      // expect(symbol).toBe(tokenInfo.symbol);
      // expect(decimals.toNumber()).toEqual(tokenInfo.decimals);
      // expect(totalSupply.toNumber()).toEqual(tokenInfo.totalSupply);
      // expect(await cep18.eventsMode()).toEqual('CES');
      // expect(isMintAndBurnEnabled).toEqual(true);
    });

    xit('should owner owns totalSupply amount of tokens', async () => {
      const balance = await cep18.balanceOf(owner.publicKey);
      expect(balance.toNumber()).toEqual(tokenInfo.totalSupply);
    });

    xit('should return 0 when balance info not found from balances dictionary', async () => {
      const consoleWarnMock = jest.spyOn(console, 'warn').mockImplementation();

      const balance = await cep18.balanceOf(user1.publicKey);

      expect(console.warn).toHaveBeenCalledWith(
        `Not found balance for ${encodeBase16(user1.publicKey.value())}`
      );
      consoleWarnMock.mockRestore();

      expect(balance.toNumber()).toEqual(0);
    });

    xit('should return 0 when allowances info not found and log warning', async () => {
      const consoleWarnMock = jest.spyOn(console, 'warn').mockImplementation();

      const allowances = await cep18.allowances(
        owner.publicKey,
        user1.publicKey
      );

      expect(console.warn).toHaveBeenCalledWith(
        `Not found allowances for ${encodeBase16(owner.publicKey.value())}`
      );
      consoleWarnMock.mockRestore();

      expect(allowances.toNumber()).toEqual(0);
    });

    xit('should throw error when try to transfer more than owned balance', async () => {
      const amount = 5_000_000_000_000;
      const deploy = cep18.transfer(
        { recipient: user1.publicKey, amount },
        5_000_000_000,
        owner.publicKey,
        NETWORK_NAME,
        [owner]
      );
      await deploy.send(NODE_URL);
      await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
      await expect(
        cep18.parseDeployResult(encodeBase16(deploy.hash))
      ).rejects.toThrowError('InsufficientBalance');
    });

    xit('should transfer tokens', async () => {
      const amount = 50;
      await doApprove(cep18, user3.publicKey, amount);

      const transferAmount = 20;

      expect(
        await transferToken(
          cep18,
          owner,
          user3,
          transferAmount,
          5_000_000,
          client
        )
      ).toBe(true);

      const balance = await cep18.balanceOf(user3.publicKey);

      expect(balance.toNumber()).toEqual(transferAmount);
    });

    xit('should tranfer tokens by allowances', async () => {
      const amount = 500;
      await doApprove(cep18, user1.publicKey, amount);

      const transferAmount = 20;

      const deploy = cep18.transferFrom(
        {
          owner: owner.publicKey,
          recipient: user2.publicKey,
          amount: transferAmount
        },
        5_000_000_000,
        user1.publicKey,
        NETWORK_NAME,
        [user1]
      );

      await deploy.send(NODE_URL);

      const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
      if (!result || !client.isDeploySuccessfull(result)) {
        fail('Transfer deploy failed');
      }
      await sleep(5000);
      const balance = await cep18.balanceOf(user2.publicKey);

      expect(balance.toNumber()).toEqual(transferAmount);

      const allowances = await cep18.allowances(
        owner.publicKey,
        user1.publicKey
      );

      expect(allowances).toEqual(BigNumber.from(amount).sub(transferAmount));
    });

    xit('should increaseAllowance', async () => {
      const amount = 200;
      await doApprove(cep18, user1.publicKey, amount);
      const transferAmount = 100;
      const deploy = cep18.increaseAllowance(
        {
          spender: user1.publicKey,
          amount: transferAmount
        },
        5_000_000_000,
        owner.publicKey,
        NETWORK_NAME,
        [owner]
      );
      await deploy.send(NODE_URL);
      const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
      if (!result || !client.isDeploySuccessfull(result)) {
        fail('Transfer deploy failed');
      }
      const allowances = await cep18.allowances(
        owner.publicKey,
        user1.publicKey
      );

      expect(allowances).toEqual(BigNumber.from(amount).add(transferAmount));
    });

    xit('should decreaseAllowance', async () => {
      const amount = 200;
      await doApprove(cep18, user1.publicKey, amount);
      const transferAmount = 100;
      const deploy = cep18.decreaseAllowance(
        {
          spender: user1.publicKey,
          amount: transferAmount
        },
        5_000_000_000,
        owner.publicKey,
        NETWORK_NAME,
        [owner]
      );

      await deploy.send(NODE_URL);
      const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
      if (!result || !client.isDeploySuccessfull(result)) {
        fail('Transfer deploy failed');
      }
      const allowances = await cep18.allowances(
        owner.publicKey,
        user1.publicKey
      );

      expect(allowances).toEqual(BigNumber.from(amount).sub(transferAmount));
    });
  });

  describe('User mint and burn', () => {
    const amountToMint = 100;
    const amountToBurn = 300;

    it('Admin should burn and mint', async () => {
      const cep18 = mintAndBurnTestsCep18;
      const userForTest = user1;
      const args = {
        adminList: [userForTest.publicKey]
      };
      expect(await changeSecurity(cep18, args, owner, client)).toBe(true);
      expect(await mint(cep18, userForTest, amountToMint, client)).toBe(true);
      expect(
        await transferToken(
          cep18,
          owner,
          userForTest,
          amountToBurn,
          5_000_000_000,
          client
        )
      ).toBe(true); // Non owners can't burn token that wasn't given to them
      expect(await burn(cep18, userForTest, amountToBurn, client)).toBe(true);
    });

    it('Minter should mint', async () => {
      const cep18 = mintAndBurnTestsCep18;
      const userForTest = user2;
      const args = {
        minterList: [userForTest.publicKey]
      };
      expect(await changeSecurity(cep18, args, owner, client)).toBe(true);
      expect(await mint(cep18, userForTest, amountToMint, client)).toBe(true);
      expect(await burn(cep18, userForTest, amountToBurn, client)).toBe(false);
    });

    it('Burner should burn', async () => {
      const cep18 = mintAndBurnTestsCep18;
      const userForTest = user3;
      const args = {
        burnerList: [userForTest.publicKey]
      };
      expect(await changeSecurity(cep18, args, owner, client)).toBe(true);
      expect(
        await transferToken(
          cep18,
          owner,
          userForTest,
          amountToBurn,
          5_000_000_000,
          client
        )
      ).toBe(true); // Non owners can't burn token that wasn't given to them
      expect(await burn(cep18, userForTest, amountToBurn, client)).toBe(true);
      expect(await mint(cep18, userForTest, amountToMint, client)).toBe(false);
    });

    afterAll(async () => {
      // Check that after all the tests the amount of token equals to the initial applied all the mints and burns
      const supplyAfterBurn = await mintAndBurnTestsCep18.totalSupply();
      expect(supplyAfterBurn.toNumber()).toEqual(
        initialSupply +
        amountToMint +
        amountToMint -
        amountToBurn -
        amountToBurn
      );
    });
  });
  it('Owner Mint and burn', async () => {
    const globalSupplyAfterTest = await ownerShouldMint(
      ownerMintAndBurnTestsCep18,
      initialSupply
    );
    await ownerShouldBurn(
      ownerMintAndBurnTestsCep18,
      globalSupplyAfterTest
    );
  });

  it('user should not mint if added to none list', async () => {
    const userForTest = user3;
    const amountToMint = 300;
    const args = {
      minterList: [userForTest.publicKey]
    };
    expect(
      await changeSecurity(mintAndBurnTestsCep18, args, owner, client)
    ).toBe(true);
    const args2 = {
      noneList: [userForTest.publicKey]
    };
    expect(
      await changeSecurity(mintAndBurnTestsCep18, args2, owner, client)
    ).toBe(true);
    expect(
      await mint(mintAndBurnTestsCep18, userForTest, amountToMint, client)
    ).toBe(false);
  });

  async function ownerShouldMint(
    cep18: CEP18Client,
    inputSupply: number
  ): Promise<number> {
    const amountToMint = 20;

    await mint(cep18, owner, amountToMint, client);
    const totalSupply = await cep18.totalSupply();

    expect(totalSupply.toNumber()).toEqual(inputSupply + amountToMint);
    return totalSupply.toNumber();
  }
  async function ownerShouldBurn(
    cep18: CEP18Client,
    inputSupply: number
  ): Promise<number> {
    const amountToBurn = 20;

    await burn(cep18, owner, amountToBurn, client);
    const totalSupply = await cep18.totalSupply();

    expect(totalSupply.toNumber()).toEqual(inputSupply - amountToBurn);
    return totalSupply.toNumber();
  }
});

async function transferToken(
  cep18: CEP18Client,
  from: AsymmetricKey,
  to: AsymmetricKey,
  amount: number,
  motesAmount: number,
  client: CasperServiceByJsonRPC
): Promise<boolean> {
  const deploy = cep18.transfer(
    { recipient: to.publicKey, amount },
    motesAmount,
    from.publicKey,
    NETWORK_NAME,
    [from]
  );
  await deploy.send(NODE_URL);
  const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
  if (!result || !client.isDeploySuccessfull(result)) {
    return false;
  }
  return true;
}

async function changeSecurity(
  cep18: CEP18Client,
  args: ChangeSecurityArgs,
  owner: AsymmetricKey,
  client: CasperServiceByJsonRPC
): Promise<boolean> {
  const deploy = cep18.changeSecurity(
    args,
    5_000_000_000,
    owner.publicKey,
    NETWORK_NAME,
    [owner]
  );
  await deploy.send(NODE_URL);
  const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
  if (!result || !client.isDeploySuccessfull(result)) {
    return false;
  }
  return true;
}

async function mint(
  cep18: CEP18Client,
  user: AsymmetricKey,
  amountToMint: number,
  client: CasperServiceByJsonRPC
): Promise<boolean> {
  const deploy = cep18.mint(
    {
      owner: user.publicKey,
      amount: amountToMint
    },
    5000000000,
    user.publicKey,
    NETWORK_NAME,
    [user]
  );

  await deploy.send(NODE_URL);

  const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
  if (!result || !client.isDeploySuccessfull(result)) {
    return false;
  }
  await sleep(5000);
  return true;
}

async function burn(
  cep18: CEP18Client,
  user: AsymmetricKey,
  amountToBurn: number,
  client: CasperServiceByJsonRPC
): Promise<boolean> {
  const deploy = cep18.burn(
    {
      owner: user.publicKey,
      amount: amountToBurn
    },
    5000000000,
    user.publicKey,
    NETWORK_NAME,
    [user]
  );

  await deploy.send(NODE_URL);

  const result = await client.waitForDeploy(deploy, DEPLOY_TIMEOUT);
  if (!result || !client.isDeploySuccessfull(result)) {
    return false;
  }
  await sleep(5000);
  return true;
}

// function waitAtMostForMessages(
//   cep18: CEP18Client,
//   waitMilliseconds: number
// ): SetAllowance[] {
//   const messages = [];
//   let keepTrying = true;
//   const start = Date.now();
//   cep18.on('SetAllowance', event => {
//     messages.push(event);
//   });
//   while (keepTrying) {
//     if (messages.length > 0) {
//       keepTrying = false;
//     }
//     if (Date.now() - start > waitMilliseconds) {
//       keepTrying = false;
//     }
//   }
//   return messages as SetAllowance[];
// }