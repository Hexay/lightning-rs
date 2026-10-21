# Thunder physics literature vs the radial (range) error

Searched 2026-10-13 using web search, Crossref API, Semantic Scholar (rate-limited, so few hits), and publisher and HAL pages. Several full texts were paywalled or bot-blocked: AGU, HAL, and NAP page images. Where a claim comes only from an abstract or a search snippet, the entry says so. **Derived** marks my own arithmetic.

Context in code: `src/model/propagation.rs:39` times the CG onset from `d.hypot(cg_onset_alt_km)` with a default height of 0. The amplitude "shadow" is `1/(1+(d/audible_km)^4)` and depends on range only. In the sim twin (`src/sim/mod.rs:77`), each segment's shadow is `exp(-(h/16)^3)`, where h is the segment's **horizontal** distance. **Neither the model nor the sim has a height-dependent (refractive) shadow.**

---

## 1. Where thunder is first heard from

**Few (1970). Lightning channel reconstruction from thunder measurements.** JGR 75. Method as summarised in NRC (1986), *The Earth's Electrical Environment*, ch. "Acoustic Reconstruction of Lightning Channels", https://nap.nationalacademies.org/read/898/chapter/36. The page images would not load, so this comes from a search snippet. The thunder record is cut into ~0.5 s intervals. Each interval maps to one channel source found by array cross-correlation, and the sources are then ray-traced back. The arrival time of each part of the record is set by the path length to that part of the channel. The first arrival is therefore the nearest *audible* point of the channel.
*Implication:* this supports the model's "onset = nearest point" idea. The important word is *audible*, which §2 covers.

**Few & Teer (1974). The accuracy of acoustic reconstructions of lightning channels.** JGR 79. Citation verified. I found no abstract and have nothing to report from it.

**Johnson, Arechiga, Thomas, Edens, Anderson, Johnson (2011). Imaging thunder.** GRL 38, L19807, doi:10.1029/2011GL049162. https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2011GL049162 (abstract via search). A 4-element array plus a network located thunder regions in 3D **out to 12 km** in the 1–10 and 4–40 Hz bands. Those regions correlate with LMA VHF sources. Mismatches are attributed to current pulses (thunder) versus ionisation steps (VHF).
*Implication:* thunder sources are distributed along the whole flash structure, including in-cloud parts. The paper gives no first-arrival statistics.

**Arechiga, Johnson, Edens, Thomas, Rison (2011). Acoustic localization of triggered lightning.** JGR 116, D09103, doi:10.1029/2010JD015248 (abstract only). Nearest-neighbour distance between acoustic sources and LMA sources was **185 m at <5 km** (100 m predicted by Monte Carlo) and **800 m at >6 km** (650 m predicted).
*Implication:* even with a known array and a known source time, acoustic ranging error roughly quadruples beyond 6 km. The excess over the model comes from propagation (atmosphere) and grows with range. This fits a range-dependent bias term.

**Anderson, Johnson, Arechiga, Thomas (2014). Mapping thunder sources by inverting acoustic and electromagnetic observations.** JGR-A 119, 13287–13304, doi:10.1002/2014JD021624. I read the companion NMT MS thesis (2013) in full: http://www.nmt.edu/academics/ees/theses/2013t_anderson_j.pdf. Each channel is modelled as point sources spaced 1 m apart. Each source is ray-traced, and the resulting arrivals are superposed into an "arrival function" and turned into a **power envelope**. Observed thunder is fitted as a **weighted sum of per-channel envelopes** using MCMC in the 6–12 Hz band.
- For multi-stroke flashes, inter-stroke gaps of **0.05–0.2 s** are "unresolvable" against propagation timing errors, so the **mean stroke time** is used as the single source time.
- In CG flashes the **direct ground channel produces most of the thunder**. Upper channels that formed after the return strokes produce little.
- **Wind dominates the error.** Surface wind of 3 m/s with shear 7.5×10⁻³ s⁻¹ gives an envelope misfit of 0.8 but only a 0.25 error in source amplitude. LMA location noise (10 m horizontal, 30 m vertical) has little effect.
- The thesis notes that a ground inversion would preferentially focus *low* sources onto the receivers.

*Implication:* this is the closest published analogue of the forward model here. It backs power-sum superposition and a single time per flash, and it names wind and temperature structure as the leading error.

**Gallin, Farges, Marchiano, Coulouvrat, Defer, Rison, Schulz, Nuret (2014). Acoustic characterization of lightning discharges.** HyMeX workshop abstract. https://www.aldis.at/fileadmin/userdaten/aldis/publication/2014/2_HyMeX2014_Gallin.pdf (full text read). It covers 56 well-reconstructed flashes, and acoustic sources "perfectly followed the return stroke" and line up with EUCLID strokes. **The lower return-stroke channel (0–2 km) is reconstructed perfectly for nearby flashes. For distant sources the reconstructed altitude is wrong.** Nonlinear propagation modelling shows that "propagation is greatly affected by the wind profiles near the ground for sources at lower altitudes."
*Implication:* this is direct evidence that the low channel, which sets the model's onset, is the part propagation distorts at range.

**Lacroix, Farges, Marchiano, Coulouvrat (2018). Acoustical measurement of natural lightning flashes: reconstructions and statistical analysis of energy spectra.** JGR-A 123(21), 12040–12065, doi:10.1029/2018JD028814 (abstract via Crossref). It covers 27 flashes at distances of **0.3–20 km**, from which a separation method extracts **36 return-stroke signals**. Infrasound comes mainly from return strokes. Total energy, centre frequency and bandwidth vary with distance.

**Lacroix, Coulouvrat, Marchiano, Farges, Ripoll (2019). Acoustical energy of return strokes: a comparison between a statistical model and measurements.** GRL 46, 11479–11489, doi:10.1029/2019GL085369 (abstract via Crossref). A random tortuous return-stroke model propagates sound through an *isothermal* atmosphere. Low-frequency thunder comes from tortuosity, and acoustic energy follows **two regimes, near-field and far-field, each a different power law**. The abstract gives no exponents.
*Implication:* a single 1/r amplitude law may be wrong at short range. The exponent is unverified.

**Bestard, Coulouvrat, Farges, Mlynarczyk (2023). Acoustical power of lightning flashes.** JGR-A, doi:10.1029/2023JD038714 (abstract only). It covers 78 events (−CG, +CG and IC) from HyMeX 2012 and EXAEDRE 2018. Back-propagation corrects for absorption and stratification, and forward-propagated power reproduces isolated microphones in Corsica. The abstract stresses "strong variability of lightning as sound sources."
*Implication:* the variation in per-strike loudness is real, which matches the free_amp result that loudness refit cannot separate candidate locations.

**Rusz, Chum, Baše (2021). Locating thunder source using a large-aperture micro-barometer array.** Front. Earth Sci. 9:614820, doi:10.3389/feart.2021.614820 (full text read). It used 4 microbarometers about 10 km apart, a >2 Hz high-pass and a point-source least-squares fit. Sound speed came from a −6.5 K/km lapse with **zero wind**. Of 92 flashes, the mixed IC+CG flashes had **median source altitude 3.8 km (SD 1.6)** and the pure IC flashes **4.5 km (SD 1.3)**. The typical range was 2–6 km. The farthest usable flash was at **17 km**, and pulses from beyond 20 km were too weak. Horizontal positions agreed with EUCLID with uncertainty below 1 km. A 2 °C temperature error equals about 1.2 m/s of sound speed. The dataset had no pure-CG flashes. The authors suggest infrasound comes from IC discharges or upper CG parts.
*Implication:* for low-frequency energy, the effective source is 3–5 km up, not at the channel base. This band (>2 Hz infrasound) is lower than the 20–250 Hz envelope, so the transfer is uncertain.

**Abeywardhana, Fernando, Sonnadara, Cooray (2016). Frequency analysis of thunder features.** Univ. Colombo archive, https://archive.cmb.ac.lk/items/2d490745-be2f-49f6-8521-ae3dcefa1628/full (abstract only). Thunder character depends on tortuosity, channel orientation relative to the observer and HF attenuation. The fundamental frequency is below 250 Hz. **I could not find any "Bodhika et al." thunder paper.** This Sri Lankan group paper is the nearest match. Treat the citation you named as unverified.

## 2. Audibility, refraction and the shadow zone

**Fleagle (1949). The audibility of thunder.** J. Acoust. Soc. Am. 21(4), 411–412, doi:10.1121/1.1906528. Also *Hearing thunder*, Physics Today 2(8):35, doi:10.1063/1.3066609 (search snippet). Thunder is "seldom heard at distances greater than about twenty-five kilometers." The normal lapse rate bends rays upward. For a source at about 4 km, the sound reaches the ground only within about 25 km. Wind shear near Florida storms shortens the range further on all sides.

**NRC (1986), ch. "Refraction"**, https://nap.nationalacademies.org/read/898/chapter/35 (search snippet; page image did not load). Beyond the tangent-ray point, the ground receives nothing from sources on or below that ray, while higher sources can still reach it. With 30 °C at the surface, a gradient of 9.8 K/km and a source at 5 km, the shadow edge is at **25 km**. Distant lightning without thunder is attributed to thermal refraction.

**Derived (verified against the NRC numbers):** in a linear lapse Γ, rays are circles of radius R = c/|dc/dz| = **2T/Γ**, which does not depend on c. A source at height z reaches the ground out to x = √(2Rz). With T = 303 K and Γ = 9.8 K/km, R = 61.9 km, and z = 5 km gives x = 24.9 km, matching the 25 km above. Inverting this, the **lowest channel point audible at ground range D is z_min(D) = D²/(2R) = D²Γ/(4T)**:

| D (km) | z_min, Γ=6.5 K/km, T=293 K (R=90 km) | z_min, Γ=9.8 K/km (R=60 km) | extra onset delay vs base-timed onset, R=60 km |
|---|---|---|---|
| 6 | 0.20 km | 0.30 km | 0.02 s |
| 10 | 0.56 km | 0.83 km | 0.10 s |
| 15 | 1.25 km | 1.88 km | 0.34 s |
| 20 | 2.22 km | 3.33 km | 0.80 s |
| 23 | 2.94 km | 4.41 km | 1.22 s |

The delay is ≈ (√(D²+z_min²) − D)/c with c = 343 m/s. The curved-ray excess path is ignored, so the true value is slightly larger. **The delay is not constant. It grows roughly as D³**, so a fixed `cg_onset_alt_km` cannot represent it, and a global clock offset cannot absorb it.

**Wind.** The refraction literature notes that a uniform wind barely refracts sound. Its *gradient* creates the effective-sound-speed gradient (summary of the search results; Cheinet JASA 2012, https://data-ww3.ifremer.fr/BIB/Cheinet_JASA2012.pdf; NACA upwind-shadow memo, https://ntrs.nasa.gov/api/citations/19930089629/downloads/19930089629.pdf). Abstract-level only. **Derived:** the lapse term c·Γ/(2T) ≈ 5.7 m/s per km at 9.8 K/km. A shear of 2–5 m/s per km, which is typical near storms and outflows (unverified), therefore changes the effective gradient by 35–90%. Downwind it can cancel the shadow almost completely. Upwind it can roughly halve R, which doubles z_min. Gallin 2014 and Anderson 2014 both single out near-ground wind as the main error for low sources.

**Audibility rules of thumb** (search snippets, low rigour): the WMO Cloud Atlas says thunder is usually not heard beyond about 20 km, and the AMS glossary says about 10 mi is typical and about 25 mi is the upper limit. **Farges et al. (2021), Atmosphere 12(9):1188, doi:10.3390/atmos12091188** (abstract via Crossref): infrasound amplitude at 0.5–5 Hz decays as **d^−0.717 within 100 km**. That is infrasound only and does not transfer to audible thunder.

## 3. Lightning-network accuracy and classification

- **Schulz et al. (2016). EUCLID Part 1.** NHESS 16:595, https://nhess.copernicus.org/articles/16/595/2016/ (abstract). Median location accuracy is **about 100 m in the network centre and better than 500 m over most of the network**. Ground truth came from Gaisberg tower currents and from E-field plus video in AT, BE and FR. The abstract gives no detection-efficiency or IC/CG percentages.
- **Diendorfer et al. (2014), ILDC** (search snippet only): before the later improvements, the Gaisberg tower position lay inside the LLS **50% ellipse for 49% of return strokes**, so the ellipses were roughly calibrated.
- **NORDLIS** (FMI page, https://en.ilmatieteenlaitos.fi/thunderstorms-in-finland): a cooperative network across FI, NO, SE, EE, LV and LT using IMPACT low-frequency sensors. **Detection efficiency is >90% in most of the network and median location accuracy is about 0.5–1 km**, both best in central Sweden and western Finland. It is a web page, not a study. Mäkelä et al. (2016), *Atmos. Res.* 172–173, doi:10.1016/j.atmosres.2015.12.009, validates against tree strikes, but I could not get its numbers. A 2011 ECSS slide deck compared GLD360 with NORDLIS and found differences of mean 9.4 km and median 7.5 km. That is network disagreement, from a secondhand snippet.
- **Flash grouping** (snippet; I did not confirm the original source): strokes are grouped within 10 km of the first stroke, ≤500 ms after the previous stroke and ≤1 s after the first. Poelman et al. (2024), NHESS 24:2511, put the 99th percentile of ground-strike-point to first-stroke distance below 10 km.
- I found no verified IC/CG misclassification rates for NORDLIS or MET UALF data.

*Implication:* NORDLIS is 5–10× less accurate than EUCLID's centre. At 0.5–1 km median, catalogue position error is comparable to part of the ~2.6 km radial error, but it is random per strike and should average down. A uniform one-sided bias from the catalogue is unlikely. Multiple strokes in one flash share a channel. If each stroke counts as an independent "heard strike", n_eff and the reliability flag are inflated.

## 4. ThunderLoc (Lu, Wang, Qin, Wang 2023)

**A practice-distributed thunder-localization system with crowd-sourced smart IoT devices.** Sensors 23(9):4186, doi:10.3390/s23094186, https://pmc.ncbi.nlm.nih.gov/articles/PMC10181301/ (full text read). Each phone has two microphones and reports **one bit: the sign of its inter-microphone TDOA**. The server picks the grid cell whose bit pattern is closest by Hamming distance, with a robust Gaussian-weighted variant (σ = 1). An energy-×4 trigger fires the recording, and a neural network separates thunder from noise. In the virtual thunder test, 30 phones over 1 km² gave **RMSE 64.2 m**. The 3D simulation over 10×10 km with 80–160 phones reports precision and accuracy ">80%" with no metres given. The thunder source is assumed to be at about 3 km height. The paper gives no sound-speed or refraction modelling and no flash matching.
*Implication:* none for range error. It locates the source from many known receivers, which is the inverse problem.

## 5. Matching thunder to overlapping flashes

- Anderson 2014 treats thunder as a **weighted superposition** of channel envelopes and fits the weights by MCMC. The posterior shows correlated, ambiguous channels. This is the published form of the repo's power-sum approach.
- Farges 2021 used **one-to-one association** with WWLLN within 500 km, with an azimuth tolerance that widens with distance: none below 30 km, up to ±30° beyond 70 km (search snippet).
- Rusz 2021 searches a 70 s window after each flash and fits a point source to each pulse. It gives no explicit rule for overlapping flashes.
- I found no paper that resolves overlapping thunder from several flashes at a *single* microphone. Treat this as an open gap.

---

## Ranked model changes worth testing

1. **Range-dependent minimum audible height (refraction shadow).** Time the CG onset from z_on(D) = max(cg_onset_alt, D²Γ/(4T)) and scale amplitude by the fraction of channel above z_on, (top − z_on)/top. This replaces `1/(1+(d/20)^4)`. Use Γ ∈ {6.5, 8, 9.8} K/km as an atmosphere nuisance. The expected onset shift is 0.1 s at 10 km, 0.8 s at 20 km and 1.2 s at 23 km (table above). This is the most plausible match to the memory note that h = 4 helped the real clip (2.73→1.90 km), whose strikes are at 6–23 km. **Do this together with item 2.**
2. **Add the same height-dependent shadow to the sim twin** (`src/sim/mod.rs:77`). The current twin has no altitude dependence, so rejecting a fixed h = 4 as "overfitting" is confounded: the twin cannot reward refraction physics by construction. Re-run the twin check after the change.
3. **Wind shear by azimuth instead of a surface wind vector.** Use R_eff(az) = 2T/Γ adjusted by (du/dz)·n̂, i.e. c/(cΓ/2T − shear·n̂). Shear is a two-parameter nuisance. Keep c + u·n̂ for travel time. Gallin 2014 and Anderson 2014 rank near-ground wind as the dominant error for low sources.
4. **Collapse strokes into flashes** (10 km / 500 ms / 1 s). Use one onset and channel per flash, and count distinct flashes in `MIN_RELIABLE_STRIKES` and n_eff. Anderson 2014 found inter-stroke timing unresolvable. This changes reliability more than accuracy.
5. **IC source height prior.** Rusz 2021 found infrasound IC sources at a median of 4.5 km (SD 1.3), against `ic_alt_km` = 7. This is low confidence because the band differs. Test 4–5 km as a sensitivity check.
6. **Catalogue position σ floor of about 0.5 km** for NORDLIS-region strikes, if UALF ellipses are smaller. Low expected effect, because the errors are random and not radially biased.
7. **Near-field amplitude law** (Lacroix 2019 two regimes). The exponents are unverified, so only test this once the paper's numbers are in hand.

Not worth pursuing from this review: ThunderLoc methods, infrasound amplitude laws, and loudness-based fixes (Bestard 2023 confirms intrinsic source variability).
