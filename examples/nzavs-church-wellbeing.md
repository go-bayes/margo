# Example: Church Attendance and Wellbeing

A walkthrough using margo to scaffold a GRF study examining the effect of religious service attendance on wellbeing outcomes.

## 1. Create the project

```bash
margo init grf church-wellbeing
cd church-wellbeing
```

## 2. Configure study.toml

Edit the generated `study.toml`:

### Paths

```toml
[paths]
pull_data = "/Users/yourname/data/nzavs"
push_mods = "/Users/yourname/outputs/church-wellbeing"
```

### Exposure

```toml
[exposure]
name = "religion_church"
reverse_score = false
binary_cutpoints = [0, 1]
threshold_label = ">="
scale_range = "scale range 0-8"
```

This creates a binary exposure: attends church (≥1/month) vs does not attend (0).

### Outcomes (wellbeing template)

```toml
[outcomes]
vars = [
  "belong",
  "bodysat",
  "forgiveness",
  "gratitude",
  "hlth_fatigue",
  "kessler_latent_anxiety",
  "kessler_latent_depression",
  "lifesat",
  "meaning_purpose",
  "meaning_sense",
  "pwi",
  "rumination",
  "self_control",
  "self_esteem",
  "short_form_health",
  "support"
]
reverse_score = []
flip = [
  "hlth_fatigue",
  "kessler_latent_anxiety",
  "kessler_latent_depression",
  "rumination"
]
```

### Labels

```toml
[labels.exposure]
religion_church = "Religious Service Attendance"
religion_church_binary = "Religious Service Attendance (binary)"

[labels.outcome]
belong = "Social Belonging"
bodysat = "Body Satisfaction"
forgiveness = "Forgiveness"
gratitude = "Gratitude"
hlth_fatigue = "Fatigue (reversed)"
kessler_latent_anxiety = "Anxiety (reversed)"
kessler_latent_depression = "Depression (reversed)"
lifesat = "Life Satisfaction"
meaning_purpose = "Meaning: Purpose"
meaning_sense = "Meaning: Sense"
pwi = "Personal Wellbeing Index"
rumination = "Rumination (reversed)"
self_control = "Self Control"
self_esteem = "Self Esteem"
short_form_health = "Self-Rated Health"
support = "Social Support"

[titles]
nice_exposure_name = "Religious Service Attendance"
nice_outcome_name = "Wellbeing Outcomes"
filename_prefix = "grf_church_wellbeing"
```

## 3. Run the workflow

```r
# in R, from the project directory
source("01-data-prep.R")
source("02-wide-format.R")
source("03-causal-forest.R")
source("04-heterogeneity.R")
source("05-policy-tree.R")
```

## 4. Interpretation

The causal forest estimates heterogeneous treatment effects of church attendance on wellbeing. Key outputs:

- **ATE plot**: average effects across all outcomes
- **Qini curves**: which outcomes show reliable heterogeneity
- **Policy trees**: who benefits most from church attendance

## Notes

- The 39 standard baseline variables control for confounding
- Flip outcomes ensure all effects are in the "positive = better" direction
- Time-varying confounders handle dynamic confounding between waves
