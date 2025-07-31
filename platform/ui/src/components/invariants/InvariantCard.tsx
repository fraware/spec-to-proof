'use client';

import { useState } from 'react';
import { Invariant, InvariantStatus, Priority } from '@/types/spec-to-proof';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { Card } from '@/components/ui/Card';
import { EditInvariantModal } from './EditInvariantModal';
import { SplitInvariantModal } from './SplitInvariantModal';
import { RenameInvariantModal } from './RenameInvariantModal';
import { ConfirmDialog } from '@/components/ui/ConfirmDialog';
import { trpc } from '@/lib/trpc';
import { toast } from '@/components/ui/Toast';

interface InvariantCardProps {
  invariant: Invariant;
  onUpdate: () => void;
}

export function InvariantCard({ invariant, onUpdate }: InvariantCardProps) {
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isSplitModalOpen, setIsSplitModalOpen] = useState(false);
  const [isRenameModalOpen, setIsRenameModalOpen] = useState(false);
  const [isConfirmRejectOpen, setIsConfirmRejectOpen] = useState(false);

  const updateInvariant = trpc.invariant.update.useMutation({
    onSuccess: () => {
      toast.success('Invariant updated successfully');
      onUpdate();
    },
    onError: (error) => {
      toast.error(`Failed to update invariant: ${error.message}`);
    },
  });

  const handleStatusChange = (status: 'CONFIRMED' | 'REJECTED') => {
    updateInvariant.mutate({
      id: invariant.id,
      status,
    });
  };

  const handleReject = () => {
    handleStatusChange('REJECTED');
    setIsConfirmRejectOpen(false);
  };

  const getStatusColor = (status: InvariantStatus) => {
    switch (status) {
      case 'EXTRACTED':
        return 'extracted';
      case 'CONFIRMED':
        return 'confirmed';
      case 'REJECTED':
        return 'rejected';
      case 'PROVEN':
        return 'proven';
      case 'FAILED':
        return 'failed';
      default:
        return 'extracted';
    }
  };

  const getPriorityColor = (priority: Priority) => {
    switch (priority) {
      case 'LOW':
        return 'low';
      case 'MEDIUM':
        return 'medium';
      case 'HIGH':
        return 'high';
      case 'CRITICAL':
        return 'critical';
      default:
        return 'low';
    }
  };

  return (
    <>
      <Card className="p-6 space-y-4 animate-fade-in-up">
        <div className="flex items-start justify-between">
          <div className="flex-1 space-y-2">
            <div className="flex items-center gap-2">
              <Badge className={`status-badge ${getStatusColor(invariant.status)}`}>
                {invariant.status}
              </Badge>
              <Badge className={`priority-badge ${getPriorityColor(invariant.priority)}`}>
                {invariant.priority}
              </Badge>
              {invariant.confidenceScore >= 0.8 && (
                <Badge className="bg-green-100 text-green-800">
                  High Confidence
                </Badge>
              )}
            </div>
            
            <h3 className="text-lg font-semibold text-gray-900">
              {invariant.description}
            </h3>
            
            <div className="space-y-2">
              <div>
                <span className="text-sm font-medium text-gray-700">Formal Expression:</span>
                <pre className="mt-1 p-3 bg-gray-50 rounded-md text-sm font-mono text-gray-800 overflow-x-auto">
                  {invariant.formalExpression}
                </pre>
              </div>
              
              {invariant.naturalLanguage && (
                <div>
                  <span className="text-sm font-medium text-gray-700">Natural Language:</span>
                  <p className="mt-1 text-gray-600">{invariant.naturalLanguage}</p>
                </div>
              )}
            </div>

            {invariant.variables.length > 0 && (
              <div>
                <span className="text-sm font-medium text-gray-700">Variables:</span>
                <div className="mt-2 space-y-1">
                  {invariant.variables.map((variable, index) => (
                    <div key={index} className="flex items-center gap-2 text-sm">
                      <span className="font-mono text-blue-600">{variable.name}</span>
                      <span className="text-gray-500">:</span>
                      <span className="text-gray-700">{variable.type}</span>
                      {variable.unit && (
                        <>
                          <span className="text-gray-500">in</span>
                          <span className="text-gray-700">{variable.unit}</span>
                        </>
                      )}
                    </div>
                  ))}
                </div>
              </div>
            )}

            {invariant.tags.length > 0 && (
              <div className="flex flex-wrap gap-1">
                {invariant.tags.map((tag, index) => (
                  <Badge key={index} variant="secondary" className="text-xs">
                    {tag}
                  </Badge>
                ))}
              </div>
            )}
          </div>
        </div>

        <div className="flex items-center justify-between pt-4 border-t border-gray-200">
          <div className="flex items-center gap-2 text-sm text-gray-500">
            <span>Confidence: {Math.round(invariant.confidenceScore * 100)}%</span>
            <span>•</span>
            <span>Extracted: {invariant.extractedAt.toLocaleDateString()}</span>
          </div>

          <div className="flex items-center gap-2">
            {invariant.status === 'EXTRACTED' && (
              <>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setIsRenameModalOpen(true)}
                  disabled={updateInvariant.isLoading}
                >
                  Rename
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setIsEditModalOpen(true)}
                  disabled={updateInvariant.isLoading}
                >
                  Edit
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => setIsSplitModalOpen(true)}
                  disabled={updateInvariant.isLoading}
                >
                  Split
                </Button>
                <Button
                  size="sm"
                  variant="default"
                  onClick={() => handleStatusChange('CONFIRMED')}
                  disabled={updateInvariant.isLoading}
                >
                  Confirm
                </Button>
                <Button
                  size="sm"
                  variant="destructive"
                  onClick={() => setIsConfirmRejectOpen(true)}
                  disabled={updateInvariant.isLoading}
                >
                  Reject
                </Button>
              </>
            )}
            
            {invariant.status === 'CONFIRMED' && (
              <Button
                size="sm"
                variant="destructive"
                onClick={() => setIsConfirmRejectOpen(true)}
                disabled={updateInvariant.isLoading}
              >
                Reject
              </Button>
            )}
            
            {invariant.status === 'REJECTED' && (
              <Button
                size="sm"
                variant="default"
                onClick={() => handleStatusChange('CONFIRMED')}
                disabled={updateInvariant.isLoading}
              >
                Restore
              </Button>
            )}
          </div>
        </div>
      </Card>

      <EditInvariantModal
        invariant={invariant}
        isOpen={isEditModalOpen}
        onClose={() => setIsEditModalOpen(false)}
        onUpdate={onUpdate}
      />

      <SplitInvariantModal
        invariant={invariant}
        isOpen={isSplitModalOpen}
        onClose={() => setIsSplitModalOpen(false)}
        onUpdate={onUpdate}
      />

      <RenameInvariantModal
        invariant={invariant}
        isOpen={isRenameModalOpen}
        onClose={() => setIsRenameModalOpen(false)}
        onUpdate={onUpdate}
      />

      <ConfirmDialog
        isOpen={isConfirmRejectOpen}
        onClose={() => setIsConfirmRejectOpen(false)}
        onConfirm={handleReject}
        title="Reject Invariant"
        message="Are you sure you want to reject this invariant? This action cannot be undone."
        confirmText="Reject"
        variant="destructive"
      />
    </>
  );
} 