// Test configuration for cross-chain integration tests
import { Near, keyStores } from 'near-api-js';
import { ethers } from 'ethers';

export interface TestConfig {
  near: {
    networkId: string;
    nodeUrl: string;
    keyPath: string;
    htlcContract: string;
    solverRegistry: string;
    testAccounts: {
      sender: string;
      receiver: string;
      solver: string;
    };
  };
  ethereum: {
    networkId: number;
    rpcUrl: string;
    hubContract: string;
    escrowFactory: string;
    testAccounts: {
      sender: string;
      receiver: string;
    };
    privateKeys: {
      sender: string;
      receiver: string;
    };
  };
  timeouts: {
    nearTimeout: number; // in seconds
    baseTimeout: number; // in seconds
  };
}

export const testConfig: TestConfig = {
  near: {
    networkId: process.env.NEAR_NETWORK || 'testnet',
    nodeUrl: process.env.NEAR_NODE_URL || 'https://rpc.testnet.near.org',
    keyPath: process.env.NEAR_KEY_PATH || '~/.near-credentials/testnet',
    htlcContract: process.env.NEAR_HTLC_CONTRACT || 'fusion-htlc.testnet',
    solverRegistry: process.env.NEAR_SOLVER_REGISTRY || 'solver-registry.testnet',
    testAccounts: {
      sender: process.env.NEAR_SENDER || 'test-sender.testnet',
      receiver: process.env.NEAR_RECEIVER || 'test-receiver.testnet',
      solver: process.env.NEAR_SOLVER || 'test-solver.testnet',
    },
  },
  ethereum: {
    networkId: parseInt(process.env.ETH_NETWORK_ID || '84532'), // BASE Sepolia
    rpcUrl: process.env.ETH_RPC_URL || 'https://sepolia.base.org',
    hubContract: process.env.ETH_HUB_CONTRACT || '',
    escrowFactory: process.env.ETH_ESCROW_FACTORY || '',
    testAccounts: {
      sender: process.env.ETH_SENDER || '',
      receiver: process.env.ETH_RECEIVER || '',
    },
    privateKeys: {
      sender: process.env.ETH_SENDER_KEY || '',
      receiver: process.env.ETH_RECEIVER_KEY || '',
    },
  },
  timeouts: {
    nearTimeout: 300, // 5 minutes
    baseTimeout: 600, // 10 minutes (must be > nearTimeout)
  },
};

// Validate timeout configuration
if (testConfig.timeouts.nearTimeout >= testConfig.timeouts.baseTimeout) {
  throw new Error('NEAR timeout must be less than BASE timeout for atomicity');
}

export async function setupNearConnection(): Promise<Near> {
  const keyStore = new keyStores.UnencryptedFileSystemKeyStore(testConfig.near.keyPath);
  
  return new Near({
    networkId: testConfig.near.networkId,
    keyStore,
    nodeUrl: testConfig.near.nodeUrl,
    walletUrl: `https://wallet.${testConfig.near.networkId}.near.org`,
    helperUrl: `https://helper.${testConfig.near.networkId}.near.org`,
  });
}

export function setupEthereumProvider(): ethers.JsonRpcProvider {
  return new ethers.JsonRpcProvider(testConfig.ethereum.rpcUrl);
}

export function getEthereumSigner(privateKey: string): ethers.Wallet {
  const provider = setupEthereumProvider();
  return new ethers.Wallet(privateKey, provider);
}