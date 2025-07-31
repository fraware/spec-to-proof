import Mathlib.Data.Nat.Basic

-- Trivial arithmetic theorem
theorem trivial_add_zero (n : Nat) : n + 0 = n := by
  simp

-- Simple equality theorem
theorem trivial_reflexivity (n : Nat) : n = n := by
  rfl

-- Basic logical theorem
theorem trivial_implies (P : Prop) : P → P := by
  intro h
  exact h
