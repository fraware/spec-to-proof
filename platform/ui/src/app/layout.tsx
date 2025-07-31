import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import './globals.css';
import { TRPCProvider } from '@/components/providers/TRPCProvider';
import { ThemeProvider } from '@/components/providers/ThemeProvider';
import { Toaster } from '@/components/ui/Toaster';

const inter = Inter({ 
  subsets: ['latin'],
  variable: '--font-inter',
  display: 'swap',
});

export const metadata: Metadata = {
  title: {
    default: 'Spec-to-Proof - Formal Verification Platform',
    template: '%s | Spec-to-Proof',
  },
  description: 'Turn everyday product specs into machine-checked Lean 4 guarantees',
  keywords: ['formal verification', 'lean4', 'specifications', 'mathematics', 'proof generation'],
  authors: [{ name: 'Spec-to-Proof Team' }],
  creator: 'Spec-to-Proof Team',
  publisher: 'Spec-to-Proof',
  formatDetection: {
    email: false,
    address: false,
    telephone: false,
  },
  metadataBase: new URL(process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000'),
  openGraph: {
    type: 'website',
    locale: 'en_US',
    url: '/',
    title: 'Spec-to-Proof - Formal Verification Platform',
    description: 'Turn everyday product specs into machine-checked Lean 4 guarantees',
    siteName: 'Spec-to-Proof',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'Spec-to-Proof - Formal Verification Platform',
    description: 'Turn everyday product specs into machine-checked Lean 4 guarantees',
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-video-preview': -1,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
  verification: {
    google: process.env.NEXT_PUBLIC_GOOGLE_VERIFICATION,
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className={inter.variable}>
      <head>
        {/* Preload critical resources */}
        <link
          rel="preload"
          href="/fonts/inter-var.woff2"
          as="font"
          type="font/woff2"
          crossOrigin="anonymous"
        />
        {/* DNS prefetch for external domains */}
        <link rel="dns-prefetch" href="//fonts.googleapis.com" />
        <link rel="dns-prefetch" href="//fonts.gstatic.com" />
      </head>
      <body className="min-h-screen bg-gray-50 font-sans antialiased">
        <ThemeProvider>
          <TRPCProvider>
            <div className="relative flex min-h-screen flex-col">
              <main className="flex-1">
                {children}
              </main>
            </div>
            <Toaster />
          </TRPCProvider>
        </ThemeProvider>
      </body>
    </html>
  );
} 