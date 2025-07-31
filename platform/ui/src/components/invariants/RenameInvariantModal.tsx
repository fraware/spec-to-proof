'use client';

import { useState } from 'react';
import { Invariant } from '@/types/spec-to-proof';
import { Modal } from '@/components/ui/Modal';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { trpc } from '@/lib/trpc';
import { toast } from '@/components/ui/Toast';
import { z } from 'zod';

const renameInvariantSchema = z.object({
  description: z.string().min(1, 'Description is required'),
});

interface RenameInvariantModalProps {
  invariant: Invariant;
  isOpen: boolean;
  onClose: () => void;
  onUpdate: () => void;
}

export function RenameInvariantModal({ invariant, isOpen, onClose, onUpdate }: RenameInvariantModalProps) {
  const [description, setDescription] = useState(invariant.description);
  const [error, setError] = useState('');

  const renameInvariant = trpc.invariant.rename.useMutation({
    onSuccess: () => {
      toast.success('Invariant renamed successfully');
      onUpdate();
      onClose();
    },
    onError: (error) => {
      toast.error(`Failed to rename invariant: ${error.message}`);
    },
  });

  const validateForm = () => {
    try {
      renameInvariantSchema.parse({ description });
      setError('');
      return true;
    } catch (error) {
      if (error instanceof z.ZodError) {
        setError(error.errors[0]?.message || 'Invalid input');
      }
      return false;
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!validateForm()) {
      return;
    }

    renameInvariant.mutate({
      id: invariant.id,
      description,
    });
  };

  const handleInputChange = (value: string) => {
    setDescription(value);
    if (error) {
      setError('');
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Rename Invariant"
      size="md"
    >
      <form onSubmit={handleSubmit} className="space-y-6">
        <div>
          <label htmlFor="description" className="block text-sm font-medium text-gray-700 mb-2">
            New Description *
          </label>
          <Input
            id="description"
            value={description}
            onChange={(e) => handleInputChange(e.target.value)}
            placeholder="Enter a new description for the invariant"
            error={error}
            autoFocus
          />
        </div>

        <div className="bg-gray-50 p-4 rounded-md">
          <h4 className="font-medium text-gray-900 mb-2">Current Description</h4>
          <p className="text-gray-600 text-sm">{invariant.description}</p>
        </div>

        <div className="flex items-center justify-end gap-3 pt-6 border-t border-gray-200">
          <Button
            type="button"
            variant="outline"
            onClick={onClose}
            disabled={renameInvariant.isLoading}
          >
            Cancel
          </Button>
          <Button
            type="submit"
            disabled={renameInvariant.isLoading}
          >
            {renameInvariant.isLoading ? 'Renaming...' : 'Rename Invariant'}
          </Button>
        </div>
      </form>
    </Modal>
  );
} 