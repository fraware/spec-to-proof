'use client';

import { useState, useEffect } from 'react';
import { Invariant, InvariantStatus } from '@/types/spec-to-proof';
import { InvariantCard } from './InvariantCard';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Select } from '@/components/ui/Select';
import { Badge } from '@/components/ui/Badge';
import { trpc } from '@/lib/trpc';
import { toast } from '@/components/ui/Toast';

interface InvariantListProps {
  documentId?: string;
}

export function InvariantList({ documentId }: InvariantListProps) {
  const [filters, setFilters] = useState({
    status: '' as InvariantStatus | '',
    search: '',
    sortBy: 'extractedAt' as 'extractedAt' | 'confidenceScore' | 'priority',
    sortOrder: 'desc' as 'asc' | 'desc',
  });
  const [page, setPage] = useState(1);
  const [selectedInvariants, setSelectedInvariants] = useState<Set<string>>(new Set());

  const {
    data,
    isLoading,
    error,
    refetch,
  } = trpc.invariant.list.useQuery({
    documentId,
    status: filters.status || undefined,
    limit: 20,
    offset: (page - 1) * 20,
  });

  const createInvariantSet = trpc.invariantSet.create.useMutation({
    onSuccess: () => {
      toast.success('Invariant set created successfully');
      setSelectedInvariants(new Set());
      refetch();
    },
    onError: (error) => {
      toast.error(`Failed to create invariant set: ${error.message}`);
    },
  });

  const publishInvariantSet = trpc.invariantSet.publish.useMutation({
    onSuccess: () => {
      toast.success('Invariant set published successfully');
      refetch();
    },
    onError: (error) => {
      toast.error(`Failed to publish invariant set: ${error.message}`);
    },
  });

  useEffect(() => {
    setPage(1);
  }, [filters]);

  const handleFilterChange = (field: string, value: string) => {
    setFilters(prev => ({ ...prev, [field]: value }));
  };

  const handleSort = (field: 'extractedAt' | 'confidenceScore' | 'priority') => {
    setFilters(prev => ({
      ...prev,
      sortBy: field,
      sortOrder: prev.sortBy === field && prev.sortOrder === 'asc' ? 'desc' : 'asc',
    }));
  };

  const handleSelectInvariant = (invariantId: string, selected: boolean) => {
    setSelectedInvariants(prev => {
      const newSet = new Set(prev);
      if (selected) {
        newSet.add(invariantId);
      } else {
        newSet.delete(invariantId);
      }
      return newSet;
    });
  };

  const handleSelectAll = () => {
    if (data?.invariants) {
      const allIds = data.invariants.map(inv => inv.id);
      setSelectedInvariants(new Set(allIds));
    }
  };

  const handleDeselectAll = () => {
    setSelectedInvariants(new Set());
  };

  const handleCreateInvariantSet = () => {
    if (selectedInvariants.size === 0) {
      toast.error('Please select at least one invariant');
      return;
    }

    createInvariantSet.mutate({
      name: `Invariant Set ${new Date().toLocaleDateString()}`,
      description: `Automatically generated invariant set with ${selectedInvariants.size} invariants`,
      invariantIds: Array.from(selectedInvariants),
      sourceDocumentIds: documentId ? [documentId] : [],
    });
  };

  const getStatusCount = (status: InvariantStatus) => {
    return data?.invariants?.filter(inv => inv.status === status).length || 0;
  };

  const getTotalCount = () => {
    return data?.total || 0;
  };

  if (error) {
    return (
      <div className="text-center py-8">
        <p className="text-red-600 mb-4">Failed to load invariants</p>
        <Button onClick={() => refetch()}>Retry</Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header with stats */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold text-gray-900">Invariants</h2>
          <div className="flex items-center gap-2">
            <Badge variant="secondary">{getTotalCount()} total</Badge>
            <Badge className="status-badge extracted">{getStatusCount('EXTRACTED')} extracted</Badge>
            <Badge className="status-badge confirmed">{getStatusCount('CONFIRMED')} confirmed</Badge>
            <Badge className="status-badge rejected">{getStatusCount('REJECTED')} rejected</Badge>
          </div>
        </div>

        {/* Filters */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          <Input
            placeholder="Search invariants..."
            value={filters.search}
            onChange={(e) => handleFilterChange('search', e.target.value)}
            className="md:col-span-2"
          />
          <Select
            value={filters.status}
            onValueChange={(value) => handleFilterChange('status', value)}
          >
            <option value="">All Status</option>
            <option value="EXTRACTED">Extracted</option>
            <option value="CONFIRMED">Confirmed</option>
            <option value="REJECTED">Rejected</option>
          </Select>
          <Select
            value={`${filters.sortBy}-${filters.sortOrder}`}
            onValueChange={(value) => {
              const [sortBy, sortOrder] = value.split('-');
              setFilters(prev => ({ ...prev, sortBy: sortBy as any, sortOrder: sortOrder as any }));
            }}
          >
            <option value="extractedAt-desc">Newest First</option>
            <option value="extractedAt-asc">Oldest First</option>
            <option value="confidenceScore-desc">Highest Confidence</option>
            <option value="confidenceScore-asc">Lowest Confidence</option>
            <option value="priority-desc">Priority (High to Low)</option>
            <option value="priority-asc">Priority (Low to High)</option>
          </Select>
        </div>

        {/* Bulk actions */}
        {selectedInvariants.size > 0 && (
          <div className="flex items-center justify-between p-4 bg-blue-50 rounded-lg mb-6">
            <div className="flex items-center gap-4">
              <span className="text-sm font-medium text-blue-900">
                {selectedInvariants.size} invariant(s) selected
              </span>
              <Button
                variant="outline"
                size="sm"
                onClick={handleDeselectAll}
              >
                Deselect All
              </Button>
            </div>
            <div className="flex items-center gap-2">
              <Button
                variant="default"
                size="sm"
                onClick={handleCreateInvariantSet}
                disabled={createInvariantSet.isLoading}
              >
                {createInvariantSet.isLoading ? 'Creating...' : 'Create Invariant Set'}
              </Button>
            </div>
          </div>
        )}
      </div>

      {/* Invariant list */}
      {isLoading ? (
        <div className="space-y-4">
          {Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-white rounded-lg border border-gray-200 p-6 animate-pulse">
              <div className="h-4 bg-gray-200 rounded w-3/4 mb-4"></div>
              <div className="h-3 bg-gray-200 rounded w-1/2 mb-2"></div>
              <div className="h-3 bg-gray-200 rounded w-2/3"></div>
            </div>
          ))}
        </div>
      ) : data?.invariants && data.invariants.length > 0 ? (
        <div className="space-y-4">
          {data.invariants.map((invariant) => (
            <div key={invariant.id} className="relative">
              <input
                type="checkbox"
                checked={selectedInvariants.has(invariant.id)}
                onChange={(e) => handleSelectInvariant(invariant.id, e.target.checked)}
                className="absolute top-4 left-4 z-10"
              />
              <div className={selectedInvariants.has(invariant.id) ? 'ml-8' : ''}>
                <InvariantCard
                  invariant={invariant}
                  onUpdate={refetch}
                />
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="text-center py-12">
          <div className="text-gray-400 mb-4">
            <svg className="mx-auto h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
          </div>
          <h3 className="text-lg font-medium text-gray-900 mb-2">No invariants found</h3>
          <p className="text-gray-500">Try adjusting your filters or check back later.</p>
        </div>
      )}

      {/* Pagination */}
      {data && data.hasMore && (
        <div className="flex items-center justify-center pt-6">
          <Button
            variant="outline"
            onClick={() => setPage(prev => prev - 1)}
            disabled={page === 1}
          >
            Previous
          </Button>
          <span className="mx-4 text-sm text-gray-600">
            Page {page} of {Math.ceil(data.total / 20)}
          </span>
          <Button
            variant="outline"
            onClick={() => setPage(prev => prev + 1)}
            disabled={!data.hasMore}
          >
            Next
          </Button>
        </div>
      )}
    </div>
  );
} 