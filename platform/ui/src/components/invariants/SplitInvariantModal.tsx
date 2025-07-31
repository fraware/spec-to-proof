'use client';

import { useState } from 'react';
import { Invariant, Priority, Variable } from '@/types/spec-to-proof';
import { Modal } from '@/components/ui/Modal';
import { Button } from '@/components/ui/Button';
import { Input } from '@/components/ui/Input';
import { Textarea } from '@/components/ui/Textarea';
import { Select } from '@/components/ui/Select';
import { TagInput } from '@/components/ui/TagInput';
import { trpc } from '@/lib/trpc';
import { toast } from '@/components/ui/Toast';
import { z } from 'zod';

const splitInvariantSchema = z.object({
  invariants: z.array(z.object({
    description: z.string().min(1, 'Description is required'),
    formalExpression: z.string().min(1, 'Formal expression is required'),
    naturalLanguage: z.string().optional(),
    variables: z.array(z.object({
      name: z.string().min(1, 'Variable name is required'),
      type: z.string().min(1, 'Variable type is required'),
      description: z.string(),
      unit: z.string(),
      constraints: z.array(z.string()),
    })),
    tags: z.array(z.string()),
    priority: z.enum(['LOW', 'MEDIUM', 'HIGH', 'CRITICAL']),
  })).min(2, 'Must split into at least 2 invariants'),
});

interface SplitInvariantModalProps {
  invariant: Invariant;
  isOpen: boolean;
  onClose: () => void;
  onUpdate: () => void;
}

export function SplitInvariantModal({ invariant, isOpen, onClose, onUpdate }: SplitInvariantModalProps) {
  const [invariants, setInvariants] = useState([
    {
      description: '',
      formalExpression: '',
      naturalLanguage: '',
      variables: [] as Variable[],
      tags: [] as string[],
      priority: 'MEDIUM' as Priority,
    },
    {
      description: '',
      formalExpression: '',
      naturalLanguage: '',
      variables: [] as Variable[],
      tags: [] as string[],
      priority: 'MEDIUM' as Priority,
    },
  ]);

  const splitInvariant = trpc.invariant.split.useMutation({
    onSuccess: () => {
      toast.success('Invariant split successfully');
      onUpdate();
      onClose();
    },
    onError: (error) => {
      toast.error(`Failed to split invariant: ${error.message}`);
    },
  });

  const validateForm = () => {
    try {
      splitInvariantSchema.parse({ invariants });
      return true;
    } catch (error) {
      if (error instanceof z.ZodError) {
        toast.error('Please fill in all required fields for each invariant');
      }
      return false;
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!validateForm()) {
      return;
    }

    splitInvariant.mutate({
      id: invariant.id,
      invariants,
    });
  };

  const updateInvariant = (index: number, field: string, value: any) => {
    setInvariants(prev => prev.map((inv, i) => 
      i === index ? { ...inv, [field]: value } : inv
    ));
  };

  const addInvariant = () => {
    setInvariants(prev => [...prev, {
      description: '',
      formalExpression: '',
      naturalLanguage: '',
      variables: [],
      tags: [],
      priority: 'MEDIUM',
    }]);
  };

  const removeInvariant = (index: number) => {
    if (invariants.length > 2) {
      setInvariants(prev => prev.filter((_, i) => i !== index));
    }
  };

  const addVariable = (invariantIndex: number) => {
    updateInvariant(invariantIndex, 'variables', [
      ...invariants[invariantIndex].variables,
      {
        name: '',
        type: '',
        description: '',
        unit: '',
        constraints: [],
      },
    ]);
  };

  const updateVariable = (invariantIndex: number, variableIndex: number, field: string, value: any) => {
    const updatedVariables = [...invariants[invariantIndex].variables];
    updatedVariables[variableIndex] = { ...updatedVariables[variableIndex], [field]: value };
    updateInvariant(invariantIndex, 'variables', updatedVariables);
  };

  const removeVariable = (invariantIndex: number, variableIndex: number) => {
    const updatedVariables = invariants[invariantIndex].variables.filter((_, i) => i !== variableIndex);
    updateInvariant(invariantIndex, 'variables', updatedVariables);
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Split Invariant"
      size="xl"
    >
      <div className="space-y-6">
        <div className="bg-blue-50 p-4 rounded-md">
          <h4 className="font-medium text-blue-900 mb-2">Original Invariant</h4>
          <p className="text-blue-800 text-sm">{invariant.description}</p>
        </div>

        <form onSubmit={handleSubmit} className="space-y-6">
          {invariants.map((invariantData, index) => (
            <div key={index} className="border border-gray-200 rounded-lg p-4 space-y-4">
              <div className="flex items-center justify-between">
                <h4 className="font-medium text-gray-900">Invariant {index + 1}</h4>
                {invariants.length > 2 && (
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => removeInvariant(index)}
                  >
                    Remove
                  </Button>
                )}
              </div>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Description *
                  </label>
                  <Textarea
                    value={invariantData.description}
                    onChange={(e) => updateInvariant(index, 'description', e.target.value)}
                    placeholder="Enter description for this invariant"
                    rows={2}
                  />
                </div>

                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Priority
                  </label>
                  <Select
                    value={invariantData.priority}
                    onValueChange={(value) => updateInvariant(index, 'priority', value)}
                  >
                    <option value="LOW">Low</option>
                    <option value="MEDIUM">Medium</option>
                    <option value="HIGH">High</option>
                    <option value="CRITICAL">Critical</option>
                  </Select>
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Formal Expression *
                </label>
                <Textarea
                  value={invariantData.formalExpression}
                  onChange={(e) => updateInvariant(index, 'formalExpression', e.target.value)}
                  placeholder="Enter the formal mathematical expression"
                  rows={3}
                  className="font-mono"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Natural Language Description
                </label>
                <Textarea
                  value={invariantData.naturalLanguage}
                  onChange={(e) => updateInvariant(index, 'naturalLanguage', e.target.value)}
                  placeholder="Optional natural language explanation"
                  rows={2}
                />
              </div>

              <div>
                <div className="flex items-center justify-between mb-2">
                  <label className="block text-sm font-medium text-gray-700">
                    Variables
                  </label>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={() => addVariable(index)}
                  >
                    Add Variable
                  </Button>
                </div>
                
                <div className="space-y-2">
                  {invariantData.variables.map((variable, varIndex) => (
                    <div key={varIndex} className="flex items-center gap-2 p-2 border border-gray-200 rounded">
                      <Input
                        placeholder="Name"
                        value={variable.name}
                        onChange={(e) => updateVariable(index, varIndex, 'name', e.target.value)}
                        className="flex-1"
                      />
                      <Input
                        placeholder="Type"
                        value={variable.type}
                        onChange={(e) => updateVariable(index, varIndex, 'type', e.target.value)}
                        className="flex-1"
                      />
                      <Input
                        placeholder="Unit"
                        value={variable.unit}
                        onChange={(e) => updateVariable(index, varIndex, 'unit', e.target.value)}
                        className="w-24"
                      />
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={() => removeVariable(index, varIndex)}
                      >
                        Remove
                      </Button>
                    </div>
                  ))}
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Tags
                </label>
                <TagInput
                  value={invariantData.tags}
                  onChange={(tags) => updateInvariant(index, 'tags', tags)}
                  placeholder="Add tags (press Enter to add)"
                />
              </div>
            </div>
          ))}

          <div className="flex items-center justify-between">
            <Button
              type="button"
              variant="outline"
              onClick={addInvariant}
            >
              Add Another Invariant
            </Button>

            <div className="flex items-center gap-3">
              <Button
                type="button"
                variant="outline"
                onClick={onClose}
                disabled={splitInvariant.isLoading}
              >
                Cancel
              </Button>
              <Button
                type="submit"
                disabled={splitInvariant.isLoading}
              >
                {splitInvariant.isLoading ? 'Splitting...' : 'Split Invariant'}
              </Button>
            </div>
          </div>
        </form>
      </div>
    </Modal>
  );
} 