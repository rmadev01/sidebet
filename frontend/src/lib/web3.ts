/// Web3 wallet utilities (ethers.js)
/// For now this is a stub — wire up when wallet integration is needed

let provider: any = null;
let signer: any = null;

export async function connectWallet(): Promise<string | null> {
    if (typeof window === 'undefined' || !(window as any).ethereum) {
        alert('Please install MetaMask or another Web3 wallet');
        return null;
    }
    try {
        const ethereum = (window as any).ethereum;
        const accounts = await ethereum.request({ method: 'eth_requestAccounts' });
        // Could initialize ethers.js provider here:
        // const { BrowserProvider } = await import('ethers');
        // provider = new BrowserProvider(ethereum);
        // signer = await provider.getSigner();
        return accounts[0] || null;
    } catch (e) {
        console.error('Wallet connection failed:', e);
        return null;
    }
}

export async function getWalletAddress(): Promise<string | null> {
    if (typeof window === 'undefined' || !(window as any).ethereum) return null;
    try {
        const accounts = await (window as any).ethereum.request({ method: 'eth_accounts' });
        return accounts[0] || null;
    } catch {
        return null;
    }
}

export function shortenAddress(addr: string): string {
    if (!addr) return '';
    return `${addr.slice(0, 6)}…${addr.slice(-4)}`;
}
