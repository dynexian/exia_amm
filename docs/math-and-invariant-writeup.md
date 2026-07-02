# Math and Invariant Writeup

This document defines the AMM formulas, rounding policies, and invariant behavior.

## 1. Invariant Model

The pool follows a constant-product AMM:

$$x \cdot y = k$$

Where:

- $x$ is reserve of token-in side
- $y$ is reserve of token-out side
- $k$ is the product of reserves

For swaps, fees are extracted before interacting with the curve.

## 2. Fee Model

Given input amount $\Delta x$:

$$fee_{protocol} = \left\lfloor \frac{\Delta x \cdot bps_{protocol}}{10000} \right\rfloor$$
$$fee_{lp} = \left\lfloor \frac{\Delta x \cdot bps_{lp}}{10000} \right\rfloor$$
$$\Delta x_{tradeable} = \Delta x - fee_{protocol} - fee_{lp}$$

Execution output:

$$\Delta y = \left\lfloor \frac{y \cdot \Delta x_{tradeable}}{x + \Delta x_{tradeable}} \right\rfloor$$

Protocol fee is transferred to treasury. LP fee remains in vault reserves, increasing LP-owned value over time.

## 3. LP Share Accounting

### First Liquidity Event

LP supply starts from geometric mean:

$$shares_{minted} = \left\lfloor \sqrt{\Delta a \cdot \Delta b} \right\rfloor$$

### Subsequent Liquidity Events

For existing reserves $(r_a, r_b)$ and total LP supply $S$:

$$shares_a = \left\lfloor \frac{\Delta a \cdot S}{r_a} \right\rfloor$$
$$shares_b = \left\lfloor \frac{\Delta b \cdot S}{r_b} \right\rfloor$$
$$shares_{minted} = \min(shares_a, shares_b)$$

Using `min` prevents value extraction through imbalanced deposits.

## 4. Remove Liquidity Math

For LP burn amount $L$ and total LP supply $S$:

$$amount_a = \left\lfloor \frac{L \cdot reserve_a}{S} \right\rfloor$$
$$amount_b = \left\lfloor \frac{L \cdot reserve_b}{S} \right\rfloor$$

Outputs are pro-rata and floor-rounded.

## 5. TWAP Accumulator

The implementation maintains cumulative prices in Q32.32 fixed-point form.

For elapsed time $\Delta t$:

$$priceA_{q32} = \left\lfloor \frac{reserve_b \cdot 2^{32}}{reserve_a} \right\rfloor$$
$$priceB_{q32} = \left\lfloor \frac{reserve_a \cdot 2^{32}}{reserve_b} \right\rfloor$$
$$cumA' = cumA + priceA_{q32} \cdot \Delta t$$
$$cumB' = cumB + priceB_{q32} \cdot \Delta t$$

If $\Delta t = 0$, accumulation is a no-op.

## 6. Rounding and Safety Policy

Rounding policy is always floor-based, favoring pool solvency over user overpayment.

- LP minting uses floor sqrt and floor ratios.
- Swap output uses floor division.
- Remove-liquidity outputs use floor division.

Arithmetic policy:

- Intermediate values are widened to `u128`.
- All critical operations use checked arithmetic.
- Invalid states return explicit errors.

## 7. Edge Cases and Expected Behavior

- Zero input amounts: rejected.
- Zero reserves on swap path: rejected.
- First LP mint resulting in zero shares: rejected.
- LP burn greater than total supply: rejected.
- Tradeable input after fees equals zero: rejected.

## 8. Test Coverage for Math

The repository includes pure unit tests validating:

- Initial LP floor-sqrt behavior.
- Subsequent LP min-ratio behavior.
- Swap fee split and floor output behavior.
- Remove-liquidity proportional calculations.
- TWAP no-op and accumulation behavior.

See `programs/exia_amm/src/math.rs` test module.
