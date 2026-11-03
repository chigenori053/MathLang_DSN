#!/usr/bin/env python3
"""Independent exact reference model for mathlang_dsn/quantum_real.rsn.

States are Gaussian-integer amplitude vectors divided by a positive integer
denominator. Every realification candidate is evaluated with exact
fractions.Fraction arithmetic, so the report contains no floating-point error.
The output must agree with `reason run --entry QuantumRealEvaluation`.
"""

from __future__ import annotations

import json
import sys
from fractions import Fraction
from itertools import combinations

CANDIDATES = [
    (1, "REAL_PART"),
    (2, "REALIFICATION"),
    (3, "BORN_COMPUTATIONAL"),
    (4, "RAW_EXPECTATION"),
    (5, "HERMITIAN_EXPECTATION"),
    (6, "LOCAL_BLOCH"),
    (7, "PAULI_TOMOGRAPHY"),
]

I2 = [1, 0, 0, 1]
X2 = [0, 1, 1, 0]
Y2 = [0, -1j, 1j, 0]
Z2 = [1, 0, 0, -1]
SIGMA_PLUS = [0, 1, 0, 0]
PAULI = [I2, X2, Y2, Z2]


def kron(a, b, da, db):
    d = da * db
    out = [0] * (d * d)
    for i in range(da):
        for j in range(da):
            for k in range(db):
                for l in range(db):
                    out[(i * db + k) * d + (j * db + l)] = a[i * da + j] * b[k * db + l]
    return out


def dim_matrices(d):
    if d == 2:
        locals_ = [X2, Y2, Z2]
        full = [X2, Y2, Z2]
        a = SIGMA_PLUS
    else:
        locals_ = [kron(p, I2, 2, 2) for p in PAULI[1:]] + [kron(I2, p, 2, 2) for p in PAULI[1:]]
        full = [kron(PAULI[i], PAULI[j], 2, 2) for i in range(4) for j in range(4) if i or j]
        a = kron(SIGMA_PLUS, I2, 2, 2)
    return locals_, full, a


def quad(m, v, d):
    total = 0
    for i in range(d):
        for j in range(d):
            total += v[i].conjugate() * m[i * d + j] * v[j]
    return complex(total)


def norm2(v):
    return sum(abs(x) ** 2 for x in v)


def frac(x):
    return Fraction(round(x.real) if isinstance(x, complex) else x)


def apply(candidate, state, d):
    """Return (real, values) for a state (amps: list[complex], den: int)."""
    amps, den = state
    n2 = round(norm2(amps))
    locals_, full, a = dim_matrices(d)
    if candidate == 1:
        return True, [Fraction(round(x.real), den) for x in amps]
    if candidate == 2:
        return True, [Fraction(round(c), den) for x in amps for c in (x.real, x.imag)]
    if candidate == 3:
        return True, [Fraction(round(abs(x) ** 2), n2) for x in amps]
    if candidate == 4:
        q = quad(a, amps, d)
        return round(q.imag) == 0, [Fraction(round(q.real), n2)]
    if candidate == 5:
        q = quad(a, amps, d)
        return True, [Fraction(round(q.real), n2)]
    mats = locals_ if candidate == 6 else full
    values = []
    real = True
    for m in mats:
        q = quad(m, amps, d)
        real = real and round(q.imag) == 0
        values.append(Fraction(round(q.real), n2))
    return real, values


def st(*amps, den=1):
    return ([complex(a) for a in amps], den)


SUITES = {
    2: [st(1, 0), st(0, 1), st(1, 1), st(1, -1), st(1, 1j), st(1, -1j), st(1, 2), st(2 + 1j, 1 - 1j)],
    4: [
        st(1, 0, 0, 0), st(0, 1, 0, 0), st(1, 0, 0, 1), st(1, 0, 0, -1),
        st(0, 1, 1, 0), st(0, 1, -1, 0), st(1, 1, 1, 1), st(1, 1j, 0, 0),
    ],
}
MULTIPLIERS = [1j, -1, 2, 1 + 2j, 3 + 4j]
ENSEMBLES = [
    [st(1, 0), st(0, 1)],
    [st(3, 4, den=5), st(4, -3, den=5)],
    [st(3, 4j, den=5), st(4j, 3, den=5)],
]


def equivalent(u, v):
    inner = sum(a.conjugate() * b for a, b in zip(u[0], v[0]))
    return round(abs(inner) ** 2) == round(norm2(u[0])) * round(norm2(v[0]))


def evaluate(candidate):
    real = invariant = separating = bounded = True
    for d, suite in SUITES.items():
        for s in suite:
            ok, base = apply(candidate, s, d)
            real = real and ok
            bounded = bounded and all(-1 <= x <= 1 for x in base)
            for c in MULTIPLIERS:
                _, scaled = apply(candidate, ([c * a for a in s[0]], s[1]), d)
                invariant = invariant and scaled == base
        for u, v in combinations(suite, 2):
            if not equivalent(u, v) and apply(candidate, u, d)[1] == apply(candidate, v, d)[1]:
                separating = False
    averages = []
    for ensemble in ENSEMBLES:
        rows = [apply(candidate, s, 2)[1] for s in ensemble]
        averages.append([sum(col) / len(rows) for col in zip(*rows)])
    ensemble_independent = all(a == averages[0] for a in averages)
    flags = [real, invariant, separating, bounded, ensemble_independent]
    score = sum(flags)
    if score == 5:
        verdict = "RECOMMENDED"
    elif real and invariant:
        verdict = "PARTIAL"
    else:
        verdict = "REJECTED"
    return {
        "id": candidate,
        "real": real,
        "ray_invariant": invariant,
        "separating": separating,
        "bounded": bounded,
        "ensemble_independent": ensemble_independent,
        "score": score,
        "verdict": verdict,
    }


def tomography_reconstructs(d):
    _, full, _ = dim_matrices(d)
    for amps, _ in SUITES[d]:
        n2 = round(norm2(amps))
        n = [round(quad(m, amps, d).real) for m in full]
        if sum(x * x for x in n) != (d - 1) * n2 * n2:
            return False
        for i in range(d):
            for j in range(d):
                rebuilt = (n2 if i == j else 0) + sum(nk * m[i * d + j] for nk, m in zip(n, full))
                if rebuilt != d * amps[i] * amps[j].conjugate():
                    return False
    return True


def main():
    reports = []
    for cid, name in CANDIDATES:
        report = evaluate(cid)
        report["name"] = name
        reports.append(report)
    out = {
        "reports": reports,
        "tomography_reconstructs_qubit": tomography_reconstructs(2),
        "tomography_reconstructs_two_qubit": tomography_reconstructs(4),
        "bell_born_phi_plus": [str(x) for x in apply(3, SUITES[4][2], 4)[1]],
        "bell_born_phi_minus": [str(x) for x in apply(3, SUITES[4][3], 4)[1]],
        "bell_xx_phi_plus": str(apply(7, SUITES[4][2], 4)[1][4]),
        "bell_xx_phi_minus": str(apply(7, SUITES[4][3], 4)[1][4]),
        "raw_expectation_counterexample_imag": quad(SIGMA_PLUS, [1, 1j], 2).imag,
    }
    json.dump(out, sys.stdout, indent=2, ensure_ascii=False)
    print()


if __name__ == "__main__":
    main()
