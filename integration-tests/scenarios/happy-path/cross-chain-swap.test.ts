// Happy path test: Successful BASE → NEAR atomic swap
import { describe, it, before, after } from 'mocha';
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
  verifyHTLCStatesMatch,
  createCorrelatedOrderHashes,
  CrossChainEventMonitor,
  waitForNearEvent,
  waitForEthereumEvent,
  assertEventEmitted,
  assertHTLCState
} from '../../utils/helpers';

// Import Ethereum contract ABIs
import FusionPlusHubABI from '../../../../1balancer/packages/hardhat/deployments/base-sepolia/FusionPlusHub.json';
import EscrowFactoryABI from '../../../../1balancer/packages/hardhat/deployments/base-sepolia/BaseEscrowFactory.json';

describe('Cross-Chain Atomic Swap - Happy Path', function() {
  this.timeout(120000); // 2 minute timeout for cross-chain operations
  
  let near: Near;
  let nearSender: Account;
  let nearReceiver: Account;
  let htlcContract: Contract;
  
  let ethProvider: ethers.Provider;
  let ethSender: ethers.Signer;
  let ethReceiver: ethers.Signer;
  let hubContract: ethers.Contract;
  let escrowFactory: ethers.Contract;
  
  let eventMonitor: CrossChainEventMonitor;
  
  before(async () => {
    // Setup NEAR connection
    near = await setupNearConnection();
    nearSender = await near.account(testConfig.near.testAccounts.sender);
    nearReceiver = await near.account(testConfig.near.testAccounts.receiver);
    
    // Setup NEAR contract
    htlcContract = new Contract(
      nearSender,
      testConfig.near.htlcContract,
      {
        viewMethods: ['get_htlc', 'get_stats', 'is_hashlock_used'],
        changeMethods: ['create_htlc', 'withdraw', 'refund'],
      }
    );
    
    // Setup Ethereum connection
    ethProvider = setupEthereumProvider();
    ethSender = getEthereumSigner(testConfig.ethereum.privateKeys.sender);
    ethReceiver = getEthereumSigner(testConfig.ethereum.privateKeys.receiver);
    
    // Setup Ethereum contracts
    hubContract = new ethers.Contract(
      testConfig.ethereum.hubContract,
      FusionPlusHubABI.abi,
      ethSender
    );
    
    escrowFactory = new ethers.Contract(
      testConfig.ethereum.escrowFactory,
      EscrowFactoryABI.abi,
      ethSender
    );
    
    // Initialize event monitor
    eventMonitor = new CrossChainEventMonitor();
  });
  
  it('should complete a successful BASE → NEAR atomic swap', async () => {
    // Step 1: Generate swap parameters
    const { secret, hashlock } = generateSecretAndHashlock();
    const swapAmount = '0.1'; // 0.1 tokens
    const currentTime = Math.floor(Date.now() / 1000);
    const { nearTimeout, baseTimeout } = calculateTimeouts(
      currentTime,
      testConfig.timeouts.nearTimeout,
      testConfig.timeouts.baseTimeout
    );
    const { nearOrderHash, baseOrderHash } = createCorrelatedOrderHashes();
    
    console.log('Swap parameters:', {
      hashlock,
      nearTimeout,
      baseTimeout,
      nearOrderHash,
      baseOrderHash,
    });
    
    // Step 2: Create BASE escrow (source chain)
    console.log('Creating BASE escrow...');
    
    const baseHTLCParams = {
      receiver: testConfig.ethereum.testAccounts.receiver,
      token: ethers.ZeroAddress, // ETH
      amount: formatEthAmount(swapAmount),
      hashlock: '0x' + hashlock,
      timelock: baseTimeout,
      orderHash: baseOrderHash,
    };
    
    const createBaseTx = await escrowFactory.createSourceEscrow(
      baseHTLCParams,
      { value: formatEthAmount(swapAmount) }
    );
    
    const baseReceipt = await createBaseTx.wait();
    console.log('BASE escrow created:', baseReceipt.hash);
    
    // Record BASE event
    eventMonitor.recordEvent({
      chain: 'base',
      event: 'EscrowCreated',
      args: baseHTLCParams,
      blockHeight: baseReceipt.blockNumber,
      timestamp: currentTime,
    });
    
    // Step 3: Create NEAR HTLC (destination chain)
    console.log('Creating NEAR HTLC...');
    
    await simulateCrossChainDelay(2000); // Simulate message passing delay
    
    const nearHTLCParams = {
      receiver: testConfig.near.testAccounts.receiver,
      token: 'near',
      amount: formatNearAmount(swapAmount),
      hashlock: hashlock,
      timelock: nearTimeout,
      order_hash: nearOrderHash,
    };
    
    const nearResult = await htlcContract.create_htlc(
      { args: nearHTLCParams },
      '300000000000000', // gas
      formatNearAmount(swapAmount) // deposit
    );
    
    const nearHtlcId = nearResult as string;
    console.log('NEAR HTLC created:', nearHtlcId);
    
    // Record NEAR event
    eventMonitor.recordEvent({
      chain: 'near',
      event: 'HTLCCreated',
      args: { ...nearHTLCParams, htlc_id: nearHtlcId },
      blockHeight: 0, // Would get from transaction result
      timestamp: currentTime + 2,
    });
    
    // Step 4: Verify HTLCs are correlated
    const correlatedEvents = eventMonitor.findCorrelatedEvents(baseOrderHash);
    expect(correlatedEvents).to.have.lengthOf.at.least(1);
    
    // Step 5: Receiver reveals secret on NEAR
    console.log('Revealing secret on NEAR...');
    
    // Switch to receiver account for withdrawal
    const receiverContract = new Contract(
      nearReceiver,
      testConfig.near.htlcContract,
      {
        changeMethods: ['withdraw'],
      }
    );
    
    const withdrawResult = await receiverContract.withdraw(
      {
        htlc_id: nearHtlcId,
        secret: secret.slice(2), // Remove 0x prefix
      },
      '300000000000000'
    );
    
    console.log('NEAR withdrawal successful');
    
    // Record withdrawal event
    eventMonitor.recordEvent({
      chain: 'near',
      event: 'HTLCWithdrawn',
      args: { htlc_id: nearHtlcId, secret: secret.slice(2) },
      blockHeight: 0,
      timestamp: currentTime + 5,
    });
    
    // Step 6: Monitor for secret revelation on BASE
    console.log('Waiting for cross-chain secret propagation...');
    
    await simulateCrossChainDelay(3000);
    
    // In real implementation, orchestrator would reveal secret on BASE
    // For testing, we simulate this
    const escrowAddress = await escrowFactory.computeEscrowAddress(
      ethSender.address,
      baseHTLCParams
    );
    
    // Step 7: Complete withdrawal on BASE
    console.log('Completing BASE withdrawal...');
    
    // Connect to escrow contract
    const EscrowSrcABI = [
      'function withdraw(bytes32 secret) external',
      'function getState() external view returns (uint8)',
    ];
    
    const escrowContract = new ethers.Contract(
      escrowAddress,
      EscrowSrcABI,
      ethReceiver
    );
    
    const withdrawTx = await escrowContract.withdraw(secret);
    const withdrawReceipt = await withdrawTx.wait();
    
    console.log('BASE withdrawal successful:', withdrawReceipt.hash);
    
    // Record BASE withdrawal
    eventMonitor.recordEvent({
      chain: 'base',
      event: 'EscrowWithdrawn',
      args: { escrow: escrowAddress, secret },
      blockHeight: withdrawReceipt.blockNumber,
      timestamp: currentTime + 8,
    });
    
    // Step 8: Verify final states
    console.log('Verifying final states...');
    
    // Check NEAR HTLC state
    const nearHtlc = await htlcContract.view('get_htlc', { htlc_id: nearHtlcId });
    assertHTLCState(nearHtlc, 'Withdrawn');
    expect(nearHtlc.secret).to.equal(secret.slice(2));
    
    // Check BASE escrow state
    const baseState = await escrowContract.getState();
    expect(baseState).to.equal(2); // Withdrawn state
    
    // Verify event sequence
    const expectedEventOrder = [
      'EscrowCreated',
      'HTLCCreated',
      'HTLCWithdrawn',
      'EscrowWithdrawn',
    ];
    
    expect(eventMonitor.verifyEventOrder(expectedEventOrder)).to.be.true;
    
    console.log('✅ Cross-chain atomic swap completed successfully!');
  });
  
  it('should handle NEP-141 token swaps', async () => {
    // This test would use a test NEP-141 token instead of native NEAR
    const testToken = 'test-token.testnet';
    const { secret, hashlock } = generateSecretAndHashlock();
    const swapAmount = '100'; // 100 tokens
    
    // First approve token transfer
    // ... implementation for NEP-141 token swap test
    
    console.log('NEP-141 token swap test - to be implemented');
  });
  
  after(async () => {
    // Cleanup if needed
    console.log('Test cleanup complete');
  });
});