import type { Metadata } from "next";
import { AppWalletProvider } from "./components/WalletProvider";
import "./globals.css";

export const metadata: Metadata = {
  title: "GhostMarket — Private Farm Marketplace",
  description: "Private farmer-to-buyer marketplace on Solana using MagicBlock",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <AppWalletProvider>
          {children}
        </AppWalletProvider>
      </body>
    </html>
  );
}