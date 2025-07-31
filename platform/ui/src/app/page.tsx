import { Suspense } from 'react';
import { InvariantList } from '@/components/invariants/InvariantList';
import { Header } from '@/components/layout/Header';
import { LoadingSpinner } from '@/components/ui/LoadingSpinner';

export default function HomePage() {
  return (
    <div className="min-h-screen bg-gray-50">
      <Header />
      
      <main className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 mb-2">
            Invariant Disambiguation Wizard
          </h1>
          <p className="text-gray-600">
            Review, edit, and manage extracted invariants before they are sent for formal verification.
          </p>
        </div>

        <Suspense fallback={<LoadingSpinner />}>
          <InvariantList />
        </Suspense>
      </main>
    </div>
  );
} 