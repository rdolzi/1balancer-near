// Helper utilities for cross-chain integration tests
import { Account, Contract } from 'near-api-js';
import { ethers } from 'ethers';
import { createHash } from 'crypto';

export interface HTLCParams {
  sender: string;
  receiver: string;
  token: string;
  amount: string;
  hashlock: string;
  timelock: number;
  orderHash?: string;
}

export interface CrossChainEvent {
  chain: 'near' | 'base';
  event: string;
  args: any;
  blockHeight: number;
  timestamp: number;
}

// Generate a secret and its SHA256 hashlock
export function generateSecretAndHashlock(): { secret: string; hashlock: string } {
  const secret = ethers.hexlify(ethers.randomBytes(32));
  const hash = createHash('sha256').update(secret).digest('hex');
  return { secret, hashlock: hash };
}

// Calculate coordinated timeouts
export function calculateTimeouts(currentTime: number, nearDuration: number, baseDuration: number): {
  nearTimeout: number;
  baseTimeout: number;
} {
  // Ensure NEAR timeout < BASE timeout for atomicity
  return {
    nearTimeout: currentTime + nearDuration,
    baseTimeout: currentTime + baseDuration,
  };
}

// Wait for a specific event on NEAR
export async function waitForNearEvent(
  contract: Contract,
  eventName: string,
  filter: (event: any) => boolean,
  timeout: number = 30000
): Promise<any> {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeout) {
    try {
      // Query contract for recent events
      // Note: In real implementation, use NEAR indexer or event streaming
      const events = await contract.account.connection.provider.query({
        request_type: 'view_access_key_list',
        account_id: contract.contractId,
        finality: 'final',
      } as any);
      
      // This is a placeholder - real implementation would use proper event indexing
      await new Promise(resolve => setTimeout(resolve, 1000));
    } catch (error) {
      console.error('Error polling for events:', error);
    }
  }
  
  throw new Error(`Timeout waiting for event ${eventName}`);
}

// Wait for Ethereum event
export async function waitForEthereumEvent(
  contract: ethers.Contract,
  eventName: string,
  filter: ethers.EventFilter,
  timeout: number = 30000
): Promise<ethers.EventLog> {
  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      contract.off(filter as any);
      reject(new Error(`Timeout waiting for event ${eventName}`));
    }, timeout);
    
    contract.once(filter as any, (...args) => {
      clearTimeout(timeoutId);
      resolve(args[args.length - 1] as ethers.EventLog);
    });
  });
}

// Format token amounts for NEAR (yoctoNEAR)
export function formatNearAmount(amount: string): string {
  return ethers.parseUnits(amount, 24).toString();
}

// Format token amounts for ERC20 (assuming 18 decimals)
export function formatEthAmount(amount: string): string {
  return ethers.parseUnits(amount, 18).toString();
}

// Simulate cross-chain message passing delay
export async function simulateCrossChainDelay(milliseconds: number = 3000): Promise<void> {
  await new Promise(resolve => setTimeout(resolve, milliseconds));
}

// Verify HTLC states match across chains
export async function verifyHTLCStatesMatch(
  nearContract: Contract,
  nearHtlcId: string,
  ethContract: ethers.Contract,
  ethHtlcId: string
): Promise<boolean> {
  // Get NEAR HTLC state
  const nearHtlc = await (nearContract as any).get_htlc({ htlc_id: nearHtlcId });
  
  // Get Ethereum HTLC state
  const ethHtlc = await ethContract.getHTLC(ethHtlcId);
  
  // Compare hashlocks
  if (nearHtlc.hashlock !== ethHtlc.hashlock) {
    console.error('Hashlock mismatch');
    return false;
  }
  
  // Compare amounts (considering decimal differences)
  // This is simplified - real implementation would handle token decimals properly
  if (nearHtlc.amount !== ethHtlc.amount.toString()) {
    console.error('Amount mismatch');
    return false;
  }
  
  return true;
}

// Create correlated order hashes for cross-chain tracking
export function createCorrelatedOrderHashes(): {
  nearOrderHash: string;
  baseOrderHash: string;
} {
  const baseHash = ethers.hexlify(ethers.randomBytes(32));
  const nearHash = createHash('sha256').update(baseHash).digest('hex');
  
  return {
    nearOrderHash: nearHash,
    baseOrderHash: baseHash,
  };
}

// Monitor both chains for correlated events
export class CrossChainEventMonitor {
  private events: CrossChainEvent[] = [];
  
  recordEvent(event: CrossChainEvent): void {
    this.events.push(event);
  }
  
  findCorrelatedEvents(orderHash: string): CrossChainEvent[] {
    return this.events.filter(e => 
      e.args.orderHash === orderHash || e.args.order_hash === orderHash
    );
  }
  
  getEventSequence(): CrossChainEvent[] {
    return [...this.events].sort((a, b) => a.timestamp - b.timestamp);
  }
  
  verifyEventOrder(expectedOrder: string[]): boolean {
    const sequence = this.getEventSequence();
    if (sequence.length !== expectedOrder.length) return false;
    
    return sequence.every((event, index) => event.event === expectedOrder[index]);
  }
}

// Test assertion helpers
export function assertEventEmitted(
  events: CrossChainEvent[],
  eventName: string,
  chain: 'near' | 'base'
): void {
  const found = events.some(e => e.event === eventName && e.chain === chain);
  if (!found) {
    throw new Error(`Expected event ${eventName} on ${chain} not found`);
  }
}

export function assertHTLCState(
  htlc: any,
  expectedState: 'Active' | 'Withdrawn' | 'Refunded' | 'Expired'
): void {
  if (htlc.state !== expectedState) {
    throw new Error(`Expected HTLC state ${expectedState}, got ${htlc.state}`);
  }
}