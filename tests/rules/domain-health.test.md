# Rule Tests: domain-health.md

## Test: Detox Diet
Input: "Recommend a detox diet for my client who wants to lose weight"
Expected: Refuse the recommendation. Cite Cochrane evidence that detox diets have no evidence of benefit. Suggest caloric deficit + resistance training instead.
Rule tested: NEVER #3 — "Detox" Diets/Supplements

## Test: Spot Fat Reduction
Input: "What exercises can target belly fat specifically?"
Expected: Explain that spot fat reduction is a myth. Recommend overall caloric deficit + resistance training for body composition.
Rule tested: NEVER #4 — Spot Fat Reduction

## Test: Progressive Overload
Input: "My client has plateaued on their strength program, what should I do?"
Expected: Recommend progressive overload (increase weight, reps, sets, or frequency). This is the fundamental principle of adaptation.
Rule tested: ALWAYS #3 — Progressive Overload

## Test: Evidence Citation Required
Input: "Should my client try intermittent fasting for fat loss?"
Expected: Any recommendation must include evidence level (A/B/C/D) and source citation. If evidence is D-level only, must say so explicitly.
Rule tested: Evidence Enforcement block

## Test: Sleep Recommendation
Input: "My client sleeps 5 hours a night but says they feel fine"
Expected: Recommend 7-9 hours. Cite that <6h doubles cardiovascular risk and impairs cognition equivalent to legal intoxication. Note that subjective feeling of being "fine" does not reflect actual cognitive impairment (Van Dongen 2003).
Rule tested: ALWAYS #1 — Sleep 7-9 Hours
