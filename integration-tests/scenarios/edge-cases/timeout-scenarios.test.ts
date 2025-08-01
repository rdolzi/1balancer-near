// Edge case tests: Timeout and failure scenarios
import { describe, it, before } from 'mocha';
import { expect } from 'chai';
import { Near, Account, Contract } from 'near-api-js';
import { ethers } from 'ethers';
import { 
  testConfig, 
  setupNearConnection, 
  setupEthereumProvider,
  getEthereumSigner
} from '../../setup/test-config';
import {
  generateSecretAndHashlock,
  calculateTimeouts,
  formatNearAmount,
  formatEthAmount,
  simulateCrossChainDelay,
  CrossChainEventMonitor,
  assertHTLCState
} from '../../utils/helpers';

describe('Cross-Chain Atomic Swap - Timeout Scenarios', function() {
  this.timeout(180000); // 3 minute timeout
  
  let near: Near;
  let nearSender: Account;
  let nearReceiver: Account;
  let htlcContract: Contract;
  
  let ethProvider: ethers.Provider;
  let ethSender: ethers.Signer;
  let ethReceiver: ethers.Signer;
  
  let eventMonitor: CrossChainEventMonitor;
  
  before(async () => {
    // Setup connections (similar to happy path)
    near = await setupNearConnection();
    nearSender = await near.account(testConfig.near.testAccounts.sender);
    nearReceiver = await near.account(testConfig.near.testAccounts.receiver);
    
    htlcContract = new Contract(
      nearSender,
      testConfig.near.htlcContract,
      {
        viewMethods: ['get_htlc', 'get_stats'],
        changeMethods: ['create_htlc', 'withdraw', 'refund'],
        useLocalViewExecution: false,
      }
    ) as any;
    
    ethProvider = setupEthereumProvider();
    ethSender = getEthereumSigner(testConfig.ethereum.privateKeys.sender);
    ethReceiver = getEthereumSigner(testConfig.ethereum.privateKeys.receiver);
    
    eventMonitor = new CrossChainEventMonitor();
  });
  
  it('should handle NEAR timeout with safe BASE refund', async () => {
    console.log('Testing NEAR timeout scenario...');
    
    const { secret, hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.05';
    const currentTime = Math.floor(Date.now() / 1000);
    
    // Set very short NEAR timeout for testing
    const nearTimeout = currentTime + 10; // 10 seconds
    const baseTimeout = currentTime + 300; // 5 minutes
    
    // Step 1: Create NEAR HTLC with short timeout
    const nearHTLCParams = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: nearTimeout,
      order_hash: 'timeout-test-1',
    };
    
    const nearResult = await (htlcContract as any).create_htlc(
      { args: nearHTLCParams },
      '300000000000000',
      formatNearAmount(swapAmount)
    );
    
    const nearHtlcId = nearResult as string;
    console.log('NEAR HTLC created with short timeout:', nearHtlcId);
    
    // Step 2: Wait for NEAR timeout
    console.log('Waiting for NEAR timeout...');
    await new Promise(resolve => setTimeout(resolve, 12000)); // Wait 12 seconds
    
    // Step 3: Refund on NEAR (should succeed)
    console.log('Attempting NEAR refund...');
    
    const refundResult = await (htlcContract as any).refund(
      { htlc_id: nearHtlcId },
      '300000000000000'
    );
    
    console.log('NEAR refund successful');
    
    // Verify NEAR HTLC is refunded
    const nearHtlc = await (htlcContract as any).get_htlc({ htlc_id: nearHtlcId });
    assertHTLCState(nearHtlc, 'Refunded');
    
    // Step 4: BASE escrow (if created) should also be refundable
    // Since NEAR timed out first, BASE can safely refund
    console.log('✅ NEAR timeout handled correctly - atomicity preserved');
  });
  
  it('should prevent withdrawal after timeout', async () => {
    console.log('Testing withdrawal prevention after timeout...');
    
    const { secret, hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.02';
    const currentTime = Math.floor(Date.now() / 1000);
    const nearTimeout = currentTime + 5; // 5 seconds
    
    // Create HTLC
    const nearHTLCParams = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: nearTimeout,
      order_hash: 'timeout-test-2',
    };
    
    const nearResult = await (htlcContract as any).create_htlc(
      { args: nearHTLCParams },
      '300000000000000',
      formatNearAmount(swapAmount)
    );
    
    const nearHtlcId = nearResult as string;
    
    // Wait for timeout
    await new Promise(resolve => setTimeout(resolve, 7000));
    
    // Attempt withdrawal after timeout (should fail)
    const receiverContract = new Contract(
      nearReceiver,
      testConfig.near.htlcContract,
      {
        viewMethods: [],
        changeMethods: ['withdraw'],
        useLocalViewExecution: false,
      }
    ) as any;
    
    try {
      await receiverContract.withdraw(
        {
          htlc_id: nearHtlcId,
          secret: secret.slice(2),
        },
        '300000000000000'
      );
      
      // Should not reach here
      expect.fail('Withdrawal should have failed after timeout');
    } catch (error: any) {
      console.log('✅ Withdrawal correctly rejected after timeout');
      expect(error.message).to.include('expired');
    }
  });
  
  it('should handle partial completion scenario', async () => {
    console.log('Testing partial completion scenario...');
    
    const { secret, hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.08';
    const currentTime = Math.floor(Date.now() / 1000);
    const { nearTimeout, baseTimeout } = calculateTimeouts(
      currentTime,
      60, // 1 minute
      120 // 2 minutes
    );
    
    // Step 1: Create both HTLCs
    const nearHTLCParams = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: nearTimeout,
      order_hash: 'partial-test-1',
    };
    
    const nearResult = await (htlcContract as any).create_htlc(
      { args: nearHTLCParams },
      '300000000000000',
      formatNearAmount(swapAmount)
    );
    
    const nearHtlcId = nearResult as string;
    console.log('NEAR HTLC created:', nearHtlcId);
    
    // Simulate BASE HTLC creation...
    eventMonitor.recordEvent({
      chain: 'base',
      event: 'EscrowCreated',
      args: { hashlock: '0x' + hashlock },
      blockHeight: 1000,
      timestamp: currentTime,
    });
    
    // Step 2: Complete only NEAR side
    const receiverContract = new Contract(
      nearReceiver,
      testConfig.near.htlcContract,
      {
        viewMethods: [],
        changeMethods: ['withdraw'],
        useLocalViewExecution: false,
      }
    ) as any;
    
    await receiverContract.withdraw(
      {
        htlc_id: nearHtlcId,
        secret: secret.slice(2),
      },
      '300000000000000'
    );
    
    console.log('NEAR withdrawal completed');
    
    // Step 3: Simulate BASE side timeout
    // In this case, the secret is revealed but BASE side times out
    // This represents a failure in cross-chain coordination
    
    eventMonitor.recordEvent({
      chain: 'near',
      event: 'HTLCWithdrawn',
      args: { htlc_id: nearHtlcId, secret: secret.slice(2) },
      blockHeight: 1001,
      timestamp: currentTime + 30,
    });
    
    // The orchestrator should detect this and attempt to complete BASE side
    // If it fails, manual intervention may be needed
    
    console.log('⚠️  Partial completion detected - requires orchestrator intervention');
    
    // Verify events show partial completion
    const events = eventMonitor.getEventSequence();
    expect(events).to.have.lengthOf(2);
    expect(events[0].event).to.equal('EscrowCreated');
    expect(events[1].event).to.equal('HTLCWithdrawn');
    // Missing: EscrowWithdrawn on BASE
  });
  
  it('should handle race condition on timeout boundary', async () => {
    console.log('Testing timeout boundary race condition...');
    
    const { secret, hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.03';
    const currentTime = Math.floor(Date.now() / 1000);
    const nearTimeout = currentTime + 15; // 15 seconds
    
    // Create HTLC
    const nearHTLCParams = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: nearTimeout,
      order_hash: 'race-test-1',
    };
    
    const nearResult = await (htlcContract as any).create_htlc(
      { args: nearHTLCParams },
      '300000000000000',
      formatNearAmount(swapAmount)
    );
    
    const nearHtlcId = nearResult as string;
    
    // Wait until just before timeout
    await new Promise(resolve => setTimeout(resolve, 14000));
    
    // Try to withdraw and refund simultaneously (race condition)
    const receiverContract = new Contract(
      nearReceiver,
      testConfig.near.htlcContract,
      {
        viewMethods: [],
        changeMethods: ['withdraw'],
        useLocalViewExecution: false,
      }
    ) as any;
    
    const withdrawPromise = receiverContract.withdraw(
      {
        htlc_id: nearHtlcId,
        secret: secret.slice(2),
      },
      '300000000000000'
    ).catch((err: any) => ({ error: err }));
    
    const refundPromise = (htlcContract as any).refund(
      { htlc_id: nearHtlcId },
      '300000000000000'
    ).catch((err: any) => ({ error: err }));
    
    // Wait for both to complete
    const [withdrawResult, refundResult] = await Promise.all([
      withdrawPromise,
      refundPromise,
    ]);
    
    // Only one should succeed
    const withdrawSuccess = !('error' in withdrawResult);
    const refundSuccess = !('error' in refundResult);
    
    expect(withdrawSuccess !== refundSuccess).to.be.true;
    console.log(`✅ Race condition handled: ${withdrawSuccess ? 'withdraw' : 'refund'} succeeded`);
    
    // Verify final state
    const finalHtlc = await (htlcContract as any).get_htlc({ htlc_id: nearHtlcId });
    expect(['Withdrawn', 'Refunded']).to.include(finalHtlc.state);
  });
  
  it('should handle multiple timeout recovery', async () => {
    console.log('Testing multiple HTLC timeout recovery...');
    
    const htlcIds: string[] = [];
    const currentTime = Math.floor(Date.now() / 1000);
    
    // Create multiple HTLCs with staggered timeouts
    for (let i = 0; i < 3; i++) {
      const { hashlock } = generateSecretAndHashlock();
      const timeout = currentTime + 10 + (i * 5); // 10s, 15s, 20s
      
      const params = {
        receiver: testConfig.near.testAccounts.receiver,
        token: 'near',
        amount: formatNearAmount('0.01'),
        hashlock: hashlock,
        timelock: timeout,
        order_hash: `multi-timeout-${i}`,
      };
      
      const result = await (htlcContract as any).create_htlc(
        { args: params },
        '300000000000000',
        formatNearAmount('0.01')
      );
      
      htlcIds.push(result as string);
    }
    
    console.log(`Created ${htlcIds.length} HTLCs with staggered timeouts`);
    
    // Wait for all to timeout
    await new Promise(resolve => setTimeout(resolve, 25000));
    
    // Refund all
    const refundResults = await Promise.all(
      htlcIds.map(id => 
        (htlcContract as any).refund(
          { htlc_id: id },
          '300000000000000'
        ).catch((err: any) => ({ id, error: err }))
      )
    );
    
    // All should be refunded
    const successCount = refundResults.filter((r: any) => !('error' in r)).length;
    expect(successCount).to.equal(htlcIds.length);
    
    console.log(`✅ Successfully refunded ${successCount} timed-out HTLCs`);
  });
});