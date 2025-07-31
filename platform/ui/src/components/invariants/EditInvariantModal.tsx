'use client';

import { useState } from 'react';
import { Invariant, Priority } from '@/types/spec-to-proof';
import { Modal } from '@/components/ui/Modal';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Textarea } from '@/components/ui/Textarea';
import { Select } from '@/components/ui/Select';
import { TagInput } from '@/components/ui/TagInput';
import { trpc } from '@/lib/trpc';
import { toast } from '@/components/ui/Toast';
import { z } from 'zod';

const editInvariantSchema = z.object({
  description: z.string().min(1, 'Description is required'),
  formalExpression: z.string().min(1, 'Formal expression is required'),
  naturalLanguage: z.string().optional(),
  priority: z.enum(['LOW', 'MEDIUM', 'HIGH', 'CRITICAL']),
  tags: z.array(z.string()),
});

interface EditInvariantModalProps {
  invariant: Invariant;
  isOpen: boolean;
  onClose: () => void;
  onUpdate: () => void;
}

export function EditInvariantModal({ invariant, isOpen, onClose, onUpdate }: EditInvariantModalProps) {
  const [formData, setFormData] = useState({
    description: invariant.description,
    formalExpression: invariant.formalExpression,
    naturalLanguage: invariant.naturalLanguage || '',
    priority: invariant.priority,
    tags: invariant.tags,
  });
  const [errors, setErrors] = useState<Record<string, string>>({});

  const updateInvariant = trpc.invariant.update.useMutation({
    onSuccess: () => {
      toast.success('Invariant updated successfully');
      onUpdate();
      onClose();
    },
    onError: (error) => {
      toast.error(`Failed to update invariant: ${error.message}`);
    },
  });

  const validateForm = () => {
    try {
      editInvariantSchema.parse(formData);
      setErrors({});
      return true;
    } catch (error) {
      if (error instanceof z.ZodError) {
        const newErrors: Record<string, string> = {};
        error.errors.forEach((err) => {
          if (err.path[0]) {
            newErrors[err.path[0] as string] = err.message;
          }
        });
        setErrors(newErrors);
      }
      return false;
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!validateForm()) {
      return;
    }

    updateInvariant.mutate({
      id: invariant.id,
      description: formData.description,
      formalExpression: formData.formalExpression,
      naturalLanguage: formData.naturalLanguage || undefined,
      priority: formData.priority,
      tags: formData.tags,
    });
  };

  const handleInputChange = (field: string, value: string | string[]) => {
    setFormData(prev => ({ ...prev, [field]: value }));
    if (errors[field]) {
      setErrors(prev => ({ ...prev, [field]: '' }));
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Edit Invariant"
      size="lg"
    >
      <form onSubmit={handleSubmit} className="space-y-6">
        <div>
          <label htmlFor="description" className="block text-sm font-medium text-gray-700 mb-2">
            Description *
          </label>
          <Textarea
            id="description"
            value={formData.description}
            onChange={(e) => handleInputChange('description', e.target.value)}
            placeholder="Enter a clear description of the invariant"
            error={errors.description}
            rows={3}
          />
        </div>

        <div>
          <label htmlFor="formalExpression" className="block text-sm font-medium text-gray-700 mb-2">
            Formal Expression *
          </label>
          <Textarea
            id="formalExpression"
            value={formData.formalExpression}
            onChange={(e) => handleInputChange('formalExpression', e.target.value)}
            placeholder="Enter the formal mathematical expression"
            error={errors.formalExpression}
            rows={4}
            className="font-mono"
          />
        </div>

        <div>
          <label htmlFor="naturalLanguage" className="block text-sm font-medium text-gray-700 mb-2">
            Natural Language Description
          </label>
          <Textarea
            id="naturalLanguage"
            value={formData.naturalLanguage}
            onChange={(e) => handleInputChange('naturalLanguage', e.target.value)}
            placeholder="Optional natural language explanation"
            rows={3}
          />
        </div>

        <div>
          <label htmlFor="priority" className="block text-sm font-medium text-gray-700 mb-2">
            Priority *
          </label>
          <Select
            id="priority"
            value={formData.priority}
            onValueChange={(value) => handleInputChange('priority', value)}
            error={errors.priority}
          >
            <option value="LOW">Low</option>
            <option value="MEDIUM">Medium</option>
            <option value="HIGH">High</option>
            <option value="CRITICAL">Critical</option>
          </Select>
        </div>

        <div>
          <label htmlFor="tags" className="block text-sm font-medium text-gray-700 mb-2">
            Tags
          </label>
          <TagInput
            value={formData.tags}
            onChange={(tags) => handleInputChange('tags', tags)}
            placeholder="Add tags (press Enter to add)"
          />
        </div>

        <div className="flex items-center justify-end gap-3 pt-6 border-t border-gray-200">
          <Button
            type="button"
            variant="outline"
            onClick={onClose}
            disabled={updateInvariant.isLoading}
          >
            Cancel
          </Button>
          <Button
            type="submit"
            disabled={updateInvariant.isLoading}
          >
            {updateInvariant.isLoading ? 'Updating...' : 'Update Invariant'}
          </Button>
        </div>
      </form>
    </Modal>
  );
} 