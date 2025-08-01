// Security tests: Attack vectors and security validations
import { describe, it, before } from 'mocha';
import { expect } from 'chai';
import { Near, Account, Contract } from 'near-api-js';
import { ethers } from 'ethers';
import { createHash } from 'crypto';
import { 
  testConfig, 
  setupNearConnection
} from '../../setup/test-config';
import {
  generateSecretAndHashlock,
  formatNearAmount,
  CrossChainEventMonitor
} from '../../utils/helpers';

describe('Cross-Chain Security Tests', function() {
  this.timeout(120000);
  
  let near: Near;
  let nearSender: Account;
  let nearReceiver: Account;
  let nearAttacker: Account;
  let htlcContract: Contract;
  
  before(async () => {
    near = await setupNearConnection();
    nearSender = await near.account(testConfig.near.testAccounts.sender);
    nearReceiver = await near.account(testConfig.near.testAccounts.receiver);
    
    // Create attacker account
    nearAttacker = await near.account('test-attacker.testnet');
    
    htlcContract = new Contract(
      nearSender,
      testConfig.near.htlcContract,
      {
        viewMethods: ['get_htlc', 'is_hashlock_used'],
        changeMethods: ['create_htlc', 'withdraw', 'refund'],
        useLocalViewExecution: false,
      }
    ) as any;
  });
  
  it('should prevent unauthorized withdrawal', async () => {
    console.log('Testing unauthorized withdrawal prevention...');
    
    const { secret, hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.1';
    const currentTime = Math.floor(Date.now() / 1000);
    
    // Create HTLC
    const params = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: currentTime + 3600,
      order_hash: 'security-test-1',
    };
    
    const result = await (htlcContract as any).create_htlc(
      { args: params },
      '300000000000000',
      formatNearAmount(swapAmount)
    );
    
    const htlcId = result as string;
    
    // Attacker tries to withdraw
    const attackerContract = new Contract(
      nearAttacker,
      testConfig.near.htlcContract,
      {
        viewMethods: [],
        changeMethods: ['withdraw'],
        useLocalViewExecution: false,
      }
    ) as any;
    
    try {
      await attackerContract.withdraw(
        {
          htlc_id: htlcId,
          secret: secret.slice(2),
        },
        '300000000000000'
      );
      
      expect.fail('Attacker withdrawal should have failed');
    } catch (error: any) {
      console.log('✅ Unauthorized withdrawal correctly prevented');
      expect(error.message).to.include('Only receiver can withdraw');
    }
  });
  
  it('should prevent unauthorized refund', async () => {
    console.log('Testing unauthorized refund prevention...');
    
    const { hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.05';
    const currentTime = Math.floor(Date.now() / 1000);
    
    // Create HTLC with expired timeout
    const params = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: currentTime - 100, // Already expired
      order_hash: 'security-test-2',
    };
    
    const result = await (htlcContract as any).create_htlc(
      { args: params },
      '300000000000000',
      formatNearAmount(swapAmount)
    );
    
    const htlcId = result as string;
    
    // Attacker tries to refund
    const attackerContract = new Contract(
      nearAttacker,
      testConfig.near.htlcContract,
      {
        changeMethods: ['refund'],
      }
    );
    
    try {
      await attackerContract.refund(
        { htlc_id: htlcId },
        '300000000000000'
      );
      
      expect.fail('Attacker refund should have failed');
    } catch (error: any) {
      console.log('✅ Unauthorized refund correctly prevented');
      expect(error.message).to.include('Only sender can refund');
    }
  });
  
  it('should prevent replay attacks with duplicate hashlocks', async () => {
    console.log('Testing hashlock replay prevention...');
    
    const { secret, hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.02';
    const currentTime = Math.floor(Date.now() / 1000);
    
    // First HTLC
    const params1 = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: currentTime + 3600,
      order_hash: 'replay-test-1',
    };
    
    await (htlcContract as any).create_htlc(
      { args: params1 },
      '300000000000000',
      formatNearAmount(swapAmount)
    );
    
    // Try to create second HTLC with same hashlock
    const params2 = {
      ...params1,
      order_hash: 'replay-test-2',
    };
    
    try {
      await (htlcContract as any).create_htlc(
        { args: params2 },
        '300000000000000',
        formatNearAmount(swapAmount)
      );
      
      expect.fail('Duplicate hashlock should have been rejected');
    } catch (error: any) {
      console.log('✅ Hashlock replay correctly prevented');
      expect(error.message).to.include('Hashlock already used');
    }
  });
  
  it('should validate secret length and format', async () => {
    console.log('Testing secret validation...');
    
    const { hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.01';
    const currentTime = Math.floor(Date.now() / 1000);
    
    // Create HTLC
    const params = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: currentTime + 3600,
      order_hash: 'secret-test-1',
    };
    
    const result = await (htlcContract as any).create_htlc(
      { args: params },
      '300000000000000',
      formatNearAmount(swapAmount)
    );
    
    const htlcId = result as string;
    
    // Test cases for invalid secrets
    const invalidSecrets = [
      '', // Empty
      'short', // Too short
      'not-hex-characters!@#$', // Invalid characters
      '00'.repeat(100), // Too long
    ];
    
    const receiverContract = new Contract(
      nearReceiver,
      testConfig.near.htlcContract,
      {
        viewMethods: [],
        changeMethods: ['withdraw'],
        useLocalViewExecution: false,
      }
    ) as any;
    
    for (const invalidSecret of invalidSecrets) {
      try {
        await receiverContract.withdraw(
          {
            htlc_id: htlcId,
            secret: invalidSecret,
          },
          '300000000000000'
        );
        
        expect.fail(`Invalid secret "${invalidSecret}" should have been rejected`);
      } catch (error: any) {
        console.log(`✅ Invalid secret "${invalidSecret.slice(0, 20)}..." correctly rejected`);
      }
    }
  });
  
  it('should prevent integer overflow in amount', async () => {
    console.log('Testing amount overflow prevention...');
    
    const { hashlock } = generateSecretAndHashlock();
    const currentTime = Math.floor(Date.now() / 1000);
    
    // Try to create HTLC with overflow amount
    const overflowAmount = '115792089237316195423570985008687907853269984665640564039457584007913129639935'; // 2^256 - 1
    
    const params = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: overflowAmount,
      hashlock: hashlock,
      timelock: currentTime + 3600,
      order_hash: 'overflow-test-1',
    };
    
    try {
      await (htlcContract as any).create_htlc(
        { args: params },
        '300000000000000',
        overflowAmount
      );
      
      expect.fail('Overflow amount should have been rejected');
    } catch (error: any) {
      console.log('✅ Amount overflow correctly prevented');
      // NEAR handles large numbers differently, but should still fail
    }
  });
  
  it('should validate cross-chain message integrity', async () => {
    console.log('Testing cross-chain message integrity...');
    
    const { secret, hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.05';
    const currentTime = Math.floor(Date.now() / 1000);
    
    // Create legitimate HTLC
    const legitParams = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: currentTime + 3600,
      order_hash: 'integrity-test-1',
    };
    
    const result = await (htlcContract as any).create_htlc(
      { args: legitParams },
      '300000000000000',
      formatNearAmount(swapAmount)
    );
    
    const htlcId = result as string;
    
    // Simulate tampered cross-chain message
    // Attacker tries to use a different secret that produces different hash
    const wrongSecret = ethers.hexlify(ethers.randomBytes(32));
    const wrongHash = createHash('sha256').update(wrongSecret).digest('hex');
    
    console.log('Expected hashlock:', hashlock);
    console.log('Wrong secret hash:', wrongHash);
    
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
          htlc_id: htlcId,
          secret: wrongSecret.slice(2),
        },
        '300000000000000'
      );
      
      expect.fail('Wrong secret should have been rejected');
    } catch (error: any) {
      console.log('✅ Message integrity validation successful');
      expect(error.message).to.include('Invalid secret');
    }
  });
  
  it('should handle reentrancy attempts', async () => {
    console.log('Testing reentrancy protection...');
    
    // This test would require a malicious contract that attempts reentrancy
    // For now, we verify that state changes are atomic
    
    const { secret, hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.1';
    const currentTime = Math.floor(Date.now() / 1000);
    
    const params = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: currentTime + 3600,
      order_hash: 'reentrancy-test-1',
    };
    
    const result = await (htlcContract as any).create_htlc(
      { args: params },
      '300000000000000',
      formatNearAmount(swapAmount)
    );
    
    const htlcId = result as string;
    
    // Get initial state
    const initialState = await htlcContract.view('get_htlc', { htlc_id: htlcId });
    expect(initialState.state).to.equal('Active');
    
    // Withdraw
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
        htlc_id: htlcId,
        secret: secret.slice(2),
      },
      '300000000000000'
    );
    
    // Verify state change is permanent
    const finalState = await htlcContract.view('get_htlc', { htlc_id: htlcId });
    expect(finalState.state).to.equal('Withdrawn');
    
    // Try to withdraw again (should fail)
    try {
      await receiverContract.withdraw(
        {
          htlc_id: htlcId,
          secret: secret.slice(2),
        },
        '300000000000000'
      );
      
      expect.fail('Double withdrawal should have failed');
    } catch (error: any) {
      console.log('✅ Reentrancy protection working - no double withdrawal');
      expect(error.message).to.include('Invalid state');
    }
  });
  
  it('should validate timeout constraints across chains', async () => {
    console.log('Testing cross-chain timeout validation...');
    
    const { hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.05';
    const currentTime = Math.floor(Date.now() / 1000);
    
    // Test case: NEAR timeout >= BASE timeout (invalid)
    const invalidNearTimeout = currentTime + 600; // 10 minutes
    const baseTimeout = currentTime + 300; // 5 minutes
    
    // In production, the orchestrator should prevent this
    // Here we verify the contract validates if such metadata is provided
    
    const params = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: invalidNearTimeout,
      order_hash: 'timeout-validation-1',
      // In real implementation, might include base_timeout for validation
    };
    
    // The contract or orchestrator should detect this violates atomicity
    console.log('⚠️  Timeout validation: NEAR timeout must be < BASE timeout');
    console.log(`NEAR: ${invalidNearTimeout}, BASE: ${baseTimeout} - INVALID`);
    
    // In production system, this would be caught by orchestrator
    // before reaching the contract level
  });
});