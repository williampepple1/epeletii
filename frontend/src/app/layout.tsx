import type { Metadata } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Epeletii — Ibani Scrabble",
  description: "Multiplayer Scrabble game for the Ibani language",
  openGraph: {
    title: "Epeletii — Ibani Scrabble",
    description: "Multiplayer Scrabble game for the Ibani language",
    url: "https://game.ibani.online",
    siteName: "Epeletii",
    images: [
      {
        url: "https://game.ibani.online/og-image.jpg",
        width: 1200,
        height: 1200,
        alt: "Epeletii — Ibani Scrabble Logo",
      },
    ],
    locale: "en_US",
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: "Epeletii — Ibani Scrabble",
    description: "Multiplayer Scrabble game for the Ibani language",
    images: ["https://game.ibani.online/og-image.jpg"],
  },
  icons: {
    icon: "/logo.jpg",
    shortcut: "/logo.jpg",
    apple: "/logo.jpg",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html
      lang="en"
      className={`${geistSans.variable} ${geistMono.variable} h-full antialiased`}
    >
      <body className="min-h-full flex flex-col">{children}</body>
    </html>
  );
}
