"use client";

import {useMemo, useState} from "react";
import {ConnectionProvider, WalletProvider} from "@solana/wallet-adapter-react";
import {DEFAULT_NETWORK, type NetworkId, NETWORKS} from "../../config";
import {PhantomWalletAdapter} from "@solana/wallet-adapter-phantom";
import {SolflareWalletAdapter} from "@solana/wallet-adapter-solflare";
import {WalletModalProvider} from "@solana/wallet-adapter-react-ui";
import TerminalMint from "~/components/TerminalMint";

export default function TerminalApp() {
  const [network, setNetwork] = useState<NetworkId>(DEFAULT_NETWORK);
  const endpoint = NETWORKS[network].rpc;


  const wallets = useMemo(
    () => [new PhantomWalletAdapter(), new SolflareWalletAdapter()],
    []
  );

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        <WalletModalProvider>
          <TerminalMint network={network} setNetwork={setNetwork} rpcUrl={endpoint}/>
        </WalletModalProvider>
      </WalletProvider>
    </ConnectionProvider>
  );
}
