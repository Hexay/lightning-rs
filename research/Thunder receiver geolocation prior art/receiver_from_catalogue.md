# Receiver position/clock from catalogued sources: new analogues (beyond research/prior_art_receiver_location.md)

Searched 2026-10-20 with web search, the OpenAlex API (abstracts) and the Crossref API (metadata). "Abstract-only" means I read the abstract via OpenAlex, or only the search-result snippet, and not the full text. Items already in research/prior_art_receiver_location.md are not repeated (Sukhovich 2020, Haase 1995, sferic patents, DAS vehicle, unknown-source mic self-localisation, ENF, colsto).

## Q1. Seismology: OBS / MERMAID / station position, orientation and clock from catalogued events; geometry criteria

### Takeaway
Seismology treats receiver position, orientation and clock as separate inverse problems, each with its own known-source data (shots, teleseisms, ambient noise). None of the hits jointly inverts a single receiver's position and clock against an earthquake catalogue. The most transferable identifiability result is Bondár et al. (2004). It shows that location accuracy is predicted better by the source–receiver geometry (station count, azimuthal gap, secondary azimuthal gap) than by the formal Gaussian error ellipse, which underestimates the true error. The geometry is symmetric, so the same criteria apply when the receiver is the unknown.

### Cited Findings
- **Bondár, Myers, Engdahl & Bergman (2004), "Epicentre accuracy based on seismic network criteria", GJI 156(3), doi:10.1111/j.1365-246X.2004.02070.x** (abstract-only). Catalogue uncertainties come from linearised inversion and assume Gaussian, zero-mean, uncorrelated errors. These assumptions are "commonly violated, leading to the underestimation of true location uncertainty, especially at high confidence levels", and accuracy "is most reliably estimated by station geometry". From Monte Carlo simulation of network geometry: local locations are within 5 km at 95% confidence when there are ≥10 stations, all within 250 km, an azimuthal gap <110°, a secondary azimuthal gap <160°, and at least one station within 30 km. Beyond local distances the secondary azimuthal gap alone suffices, and error grows once it exceeds 120°. — [OpenAlex/Crossref record](https://doi.org/10.1111/j.1365-246X.2004.02070.x)
- **Syracuse, Phillips, Maceira & Begnaud (2017), "Identifying and Correcting Timing Errors at Seismic Stations in and around Iran", SRL 88(6), doi:10.1785/0220170113** (abstract-only). Travel-time residuals against catalogued events flag periods with station timing errors. Corrections are proposed for 14 stations with errors lasting a month or more. The same analysis found two stations with wrong coordinates in the International Registry of Seismograph Stations. This is a newer catalogue-residual detector of both clock error and mislocation than Haase 1995. — [doi](https://doi.org/10.1785/0220170113)
- **Nolet, Simon & Bonnieux (2024), "How Accurately Are MERMAID Seismograms Located?", SRL, doi:10.1785/0220230377** (abstract-only). MERMAID float recording positions come from linear interpolation between GPS surfacing fixes, not from the teleseisms. Mean mislocation is 99 m when a recording triggers an immediate ascent and 685 m when it is interpolated later. The equivalent time errors are heavier-tailed than Gaussian. The 95% half-width is <27 ms for immediate ascent and 222 ms for interpolated recordings, though 67% of errors are below 44 ms. — [doi](https://doi.org/10.1785/0220230377)
  - Negative result for this survey: MERMAID does **not** locate the float from catalogued earthquakes. Position comes from GPS, and the teleseisms serve tomography.
- **Liu, Liu, Xing & Li (2022), "A new method for OBS relocation using direct water-wave arrival times from a shooting line and accurate bathymetric data", Marine Geophysical Research, doi:10.1007/s11001-022-09482-0** (snippet-only; the Springer page was paywalled). OBS relocation from active shots with known positions and times. It ends in a gradient grid search around a starting depth node. On 8 OBSs in the SW Pacific it shifted positions by 50–500 m (mean 206 m). — [doi](https://doi.org/10.1007/s11001-022-09482-0)
- **Geological Survey of Canada (Haida Gwaii 2012 OBS report)** (snippet-only). Airgun shots were used to relocate each OBS, which had drifted about 200 m NW of the ship drop point on average. Geophone orientation was also determined from the airgun data. — [NRCan publication record](https://publications.gc.ca/site/eng/9.819694/publication.html)
- **US patent 8,995,222**, "System and method for accurate determination of ocean bottom seismometer positioning and timing" (snippet-only). Solves jointly for receiver location and clock drift from first-break times of known shots. This is the closest hit to this repo's position-plus-clock joint inverse, but it serves exploration seismology. — [USPTO PDF](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/8995222)
- **Zhu, Yang, Lin & You (2020), "Determining the Orientation of Ocean-Bottom Seismometers on the Seafloor and Correcting for Polarity Flipping via Polarization Analysis and Waveform Modeling", SRL, doi:10.1785/0220190239** (abstract-only). Orientation comes from teleseismic P and Rayleigh polarisation and waveform fits to synthetics. Polarity reversal is confounded with unknown orientation, which is a named identifiability ambiguity. It recovered orientation for 9 of 12 Mariana OBSs. — [doi](https://doi.org/10.1785/0220190239)
- **Ekström & Busby (2008), "Measurements of seismometer orientation at USArray Transportable Array and Backbone stations", SRL** (snippet-only via the author QC page). The horizontal components are rotated to maximise correlation between observed and predicted long-period surface waves from catalogued earthquakes. The reported anomaly is the median over events. **The QC page states estimates from fewer than 5 measurements are not reliable**, a source-count reliability flag analogous to this repo's. — [Ekström PDF (IRIS)](https://ds.iris.edu/hq/files/programs/gsn/gsnqual/Ekstrom_Busby_2008_SRL.pdf); [QC page](https://www.ldeo.columbia.edu/~ekstrom/Projects/USARRAY/POLARIZATION/index2.html)
- **Braunmiller, Nabelek & Ghods (2020), "Sensor Orientation of Iranian Broadband Seismic Stations from P-Wave Particle Motion", SRL, doi:10.1785/0220200019** (Crossref metadata only). Another catalogue-event-based receiver-metadata inverse. — [doi](https://doi.org/10.1785/0220200019)
- Clock from ambient noise, without a catalogue, for contrast: Hable et al. (2018) GJI 214(3), doi:10.1093/gji/ggy236, found linear OBS drifts of 0.2–8.8 ms/day and ~1 s jumps in 3 OBSs. Gouédard et al. (2014) BSSA, doi:10.1785/0120130157. Naranjo et al. (2024) "OCloC", Seismica, doi:10.26443/seismica.v3i1.367, uses least-squares inversion over a network with distance-based weighting for uneven noise illumination. All abstract-only. — [Hable](https://doi.org/10.1093/gji/ggy236); [Gouédard](https://doi.org/10.1785/0120130157); [Naranjo](https://doi.org/10.26443/seismica.v3i1.367)
- **Apollo lunar seismometers.** Spent S-IVB and LM ascent-stage impacts, with known mass, velocity, time and coordinates, were used as calibration sources. The Apollo 12 LM impact was 73 km from the station. When tracking of the Apollo 16 S-IVB was lost, the seismically determined impact site later proved to be ~31 km from the site found in LRO imagery. — [Wikipedia, Apollo 12 PSE](https://en.wikipedia.org/wiki/Apollo_12_Passive_Seismic_Experiment); [LROC "Rocket impacts recorded by the Apollo seismic network"](https://www.lroc.asu.edu/images/364). An Onodera et al. JpGU 2017 abstract re-analyses these impacts with updated source positions (snippet-only): [PPS08-14](https://pub-files.atlas.jp/fs/public/jpguagu2017/ver_4/abstract/en/PPS08-14.pdf)
- **Probabilistic grid-search location: NonLinLoc.** Lomax, Virieux, Volant & Berge-Thierry (2000), "Probabilistic Earthquake Location in 3D and Layered Models", in *Advances in Seismic Event Location* (Modern Approaches in Geophysics), Springer, doi:10.1007/978-94-015-9536-0_5 (metadata only; I did not read the content). This is the standard seismological grid/sampled posterior-PDF location method. — [doi](https://doi.org/10.1007/978-94-015-9536-0_5)

### Inferences
- The Bondár criteria (azimuthal gap, secondary gap, nearest-station distance, count ≥10) translate directly into a reliability flag for thunderprint, with strikes taking the place of stations. The secondary-gap criterion (gap left after dropping one source) guards against one strike carrying the solution. The current flag is count-only.
- Bondár's finding that formal Gaussian ellipses underestimate true error at high confidence matches this repo's real-vs-sim fit gap. Geometry-based empirical calibration against ground truth, as Bondár did with GT explosions, is the documented remedy.
- The Apollo 16 case (seismic site ~31 km off) is a cautionary precedent: single-station or sparse-geometry locations from known-time sources can carry large biases. This is consistent with the repo's radial-error findings.

### Gaps
- I found no published joint position-plus-clock inversion of a single seismometer against an earthquake catalogue, with grid-search posterior and credible region. The patent (US 8,995,222) and Liu 2022 use active shots.
- I did not open the Lomax 2000 or Bondár 2004 full texts. Bondár's teleseismic thresholds were cut off in the abstract.

## Q2. Infrasound / acoustics: catalogued explosions, launches, bolides, eruptions as references

### Takeaway
Infrasound uses catalogued ground-truth events, especially rocket launches, to validate network detection, azimuth estimation and location. Every hit treats the event as the unknown or the test target, never the sensor position or clock. I found no source using Hunga Tonga or launches to calibrate a sensor clock or position.

### Cited Findings
- **Pilger, Hupe, Gaebler & Ceranna (2021), "1001 Rocket Launches for Space Missions and Their Infrasonic Signature", GRL 48(8), doi:10.1029/2020GL092262** (snippet-only). Ground truth for 1001 launches from 27 spaceports (2009–mid-2020). Up to 73% were detected at IMS infrasound arrays. The open dataset holds 7637 infrasound detections from 733 launches, doi:10.25928/bgrseis_1001-ifsd. — [OpenAlex/doi](https://doi.org/10.1029/2020GL092262); [EGU21 abstract](https://meetingorganizer.copernicus.org/EGU21/EGU21-8474.html)
- McLaughlin et al. (2009, SAIC, Monitoring Research Review) used rocket launches as "ground truth" to test IMS azimuth estimation under varying atmosphere. Arrowsmith et al. (2009–2010 MRR) describe Bayesian infrasound location validated against ground truth (snippet-only). — [LDEO MRR 2009](https://www.ldeo.columbia.edu/res/pi/Monitoring/Doc/Srr_2009/PAPERS/06-02.PDF); [LDEO MRR 2010](https://www.ldeo.columbia.edu/res/pi/Monitoring/Doc/Srr_2010/PAPERS/07-03.PDF)
- Popenhagen & Garcés (2025), "Acoustic Rocket Signatures Collected by Smartphones", *Signals* 6(1), doi:10.3390/signals6010005 (metadata only). Consumer-device recordings of catalogued launches. — [doi](https://doi.org/10.3390/signals6010005)
- Hunga Tonga timing studies back-calculate the *source* onset (~04:02 UTC eruption onset; pressure pulse ~04:15 UTC; Lamb wave ~314 m/s) from station arrivals and assume station clocks are correct (snippet-only). — [ESSOAr preprint](https://essopenarchive.org/doi/full/10.1002/essoar.10511401.1)

### Inferences
- Launch catalogues (exact time and pad position) are a ready ground-truth source for testing a single-sensor "receiver from catalogue" inverse at long range. The detection rate (≤73% at IMS arrays) shows the audible-source fraction is well below 1 even for huge sources. That parallels the audible-strike count limit.

### Gaps
- I found no paper inverting for an infrasound sensor's clock or position from catalogued events, Hunga Tonga included. The search for IMS timing calibration against events was unproductive.

## Q3. Radio: sferics for single-receiver positioning/timing; signals of opportunity with catalogued emitters

### Takeaway
Single-station sferic literature locates the *lightning* from one receiver (5–10% range error). I found nothing that locates the *receiver* from catalogued sferics. The strongest structural analogue is LEO signals-of-opportunity positioning, where a stationary receiver's position and clock come from emitters with (poorly) known ephemerides. There, accuracy is driven by the number of satellites, the pass geometry and the observation duration, and ephemeris error is handled as a nuisance.

### Cited Findings
- **Kozhaya & Kassas (2023), "Positioning with Starlink LEO Satellites: A Blind Doppler Spectral Approach", IEEE VTC2023-Spring, doi:10.1109/VTC2023-Spring57618.2023.10199264** (abstract-only). A stationary receiver blindly tracked Doppler from 6 Starlink satellites over ~800 s. Nonlinear least squares from an initial guess 200 km away reached 4.3 m horizontal error. — [doi](https://doi.org/10.1109/VTC2023-Spring57618.2023.10199264)
- Neinavaie, Shadram, Kozhaya & Kassas (2022), "First Results of Differential Doppler Positioning with Unknown Starlink Satellite Signals", IEEE Aerospace Conf., doi:10.1109/AERO53065.2022.9843493 (metadata only). Kozhaya, Kanj & Kassas (2023), PLANS, doi:10.1109/PLANS53410.2023.10139969, multi-constellation (OneWeb, Starlink, …) blind beacon estimation (metadata only). El-Kouba, Kozhaya & Kassas (2026), EuCAP, doi:10.23919/EuCAP68105.2026.11612532, "Performance Assessment of DOA and Doppler Positioning Under Erroneous Starlink and OneWeb LEO Satellites Ephemeris" (metadata only), the analogue of catalogue position error. — [AERO doi](https://doi.org/10.1109/AERO53065.2022.9843493); [PLANS doi](https://doi.org/10.1109/PLANS53410.2023.10139969); [EuCAP doi](https://doi.org/10.23919/EuCAP68105.2026.11612532)
- Khalife, Neinavaie & Kassas (2022): carrier-phase tracking of 6 Starlink satellites gave 7.7 m horizontal error *with known altitude* (snippet-only). — [OSU PDF](https://ece.osu.edu/sites/default/files/2022-09/The_first_carrier_phase_tracking_and_positioning_results_with_Starlink_LEO_satellite_Signals.pdf)
- Single-station sferic location of lightning, the direction opposite to this survey. A Stanford thesis uses VLF/ELF arrival-time difference for range within 6.7% and azimuth within 1.3% of NLDN (snippet-only), [Stanford PURL](https://purl.stanford.edu/qb836zb1374). A single-VLF-receiver study reports ~4.7% mean range deviation under 3500 km (snippet-only), [Ann. Geophys. 25, 1509, 2007](https://angeo.copernicus.org/articles/25/1509/2007/).
- X-ray pulsar navigation (catalogued periodic sources, unknown receiver position): pulse-phase ambiguity creates many candidate positions. Lohan's 2021 UIUC dissertation searches wavefront intersections, and more pulsars sharply cut the candidate count (snippet-only). Yang, Guo & Yang (SPIE 7385, 2009) report search-space (grid) methods as slow when the prior position error is large (snippet-only). — [Lohan dissertation](https://www.ideals.illinois.edu/items/120994); [SPIE](https://spiedigitallibrary.org/conference-proceedings-of-spie/7385/1/The-use-of-x-ray-pulsar-based-navigation-method-for/10.1117/12.835493.full)

### Inferences
- Pulsar navigation has the same multimodality as this repo's clock-offset scan, with periodic or ambiguous correlation peaks. Its remedy, more independent sources to prune candidate modes, parallels the audible-strike count criterion.
- In LEO SoOP and GNSS, the standard identifiability metric is dilution of precision from emitter geometry (general knowledge; no source fetched this session). It is the radio counterpart of the azimuthal gap.

### Gaps
- I found no published work locating a VLF/LF receiver or synchronising its clock from a lightning catalogue (WWLLN/NLDN/Blitzortung). Beyond the known US patents, the radio analogue appears absent.

## Q4. Audio forensics: geolocating/timestamping recordings from catalogued events other than ENF

### Takeaway
I found no peer-reviewed forensic method that geolocates or timestamps a single recording by physics-based matching to catalogued events (ADS-B aircraft, bells, trains, quakes, fireworks). Audio geolocation work is classification-based (learned scene/species/landmark cues). The nearest physics analogue is acoustic Doppler tracking of an aircraft from ground microphones. Inverting that, with a known ADS-B track and an unknown microphone, would match this repo's structure, but I found no publication of it.

### Cited Findings
- Martín, Genescà, Romeu & Pàmies (2012/2013), "Aircraft tracking by means of the Acoustical Doppler Effect", *Aerospace Science and Technology*, doi:10.1016/j.ast.2012.11.011 (snippet-only). Estimates aircraft closest-approach distance, time and speed from engine-tone Doppler at ground microphones. Thesis: Martín Román (2013), "Passive acoustic method for aircraft localization", UPC, doi:10.5821/dissertation-2117-95089. — [doi](https://doi.org/10.1016/j.ast.2012.11.011)
- The AeroSonicDB (YPAD-0523) dataset logged ADS-B (dump1090) alongside aircraft audio to label recordings with ICAO code and altitude. This is ADS-B-labelled audio but not localisation. — [arXiv 2311.06368](https://arxiv.org/abs/2311.06368)
- The LocaRDS dataset notes that some aircraft broadcast ADS-B positions with offsets or delays, which bears on catalogue position error if ADS-B is used as the "catalogue". — [arXiv 2012.00116](https://arxiv.org/abs/2012.00116)
- Learned audio geolocation: a natural-sounds benchmark (species audio), [arXiv 2505.18726](https://arxiv.org/abs/2505.18726). Audio-language-model geolocation benchmark, where train horns act as cues, [arXiv 2601.03227](https://arxiv.org/abs/2601.03227). A Fraunhofer acoustic geo-tagging poster uses church bells and landmark sounds, [Fraunhofer publica](https://publica.fraunhofer.de/handle/publica/486036). All snippet-only.
- A US patent on acoustic detect-and-avoid time-aligns ADS-B with ground-array audio, for intruder detection rather than self-location (snippet-only). — [US20210225182A1](https://patents.google.com/patent/US20210225182A1/en)

### Inferences
- An "ADS-B-as-catalogue" microphone locator would combine Martín et al.'s closest-approach observables with a known track. Each flyover would give a range-and-time constraint, and several flyovers on different headings would be needed. That is the same geometry and source-count logic as thunder.

### Gaps
- I found no published ADS-B-based microphone geolocation, and no earthquake-, fireworks- or sonic-boom-based recording geolocation or chronolocation (DFRWS/FSI not specifically queried in full text). Gunshot-network (ShotSpotter-style) sensor self-survey literature was not found.

## Q5. Do any analogues use a grid-search likelihood/posterior with credible regions and a reliability/identifiability metric?

### Takeaway
Yes, but in pieces, and almost always for the *source* rather than the receiver. NonLinLoc-style grid posteriors with confidence regions are standard for earthquakes. Bondár's geometry criteria are the established reliability metric. Ekström & Busby use a minimum-measurement-count reliability rule for orientation, and OBS relocation uses grid search. None of the receiver-side analogues found here combines all three (a grid posterior over receiver position and clock, a credible region, and a geometry/count reliability flag). That combination in thunderprint appears undocumented in the literature found.

### Cited Findings
- Grid/sampled posterior location: Lomax et al. 2000, doi:10.1007/978-94-015-9536-0_5 (metadata only). — [doi](https://doi.org/10.1007/978-94-015-9536-0_5)
- Geometry-based accuracy criteria replacing formal ellipses: Bondár et al. 2004 (abstract). — [doi](https://doi.org/10.1111/j.1365-246X.2004.02070.x)
- Count-based reliability (<5 measurements unreliable): Ekström & Busby QC page. — [link](https://www.ldeo.columbia.edu/~ekstrom/Projects/USARRAY/POLARIZATION/index2.html)
- Receiver grid search with known sources: Liu et al. 2022 OBS relocation (snippet). — [doi](https://doi.org/10.1007/s11001-022-09482-0)
- Heavy-tailed rather than Gaussian receiver-position-induced timing errors, reported as 95% half-widths: Nolet et al. 2024 (abstract). — [doi](https://doi.org/10.1785/0220230377)

### Inferences
- A defensible upgrade to the reliability flag, citing Bondár 2004: report the azimuthal gap and secondary azimuthal gap of the *audible* strikes, plus the nearest-audible-strike distance, alongside the count. Calibrate thresholds by Monte Carlo on the sim suite, as Bondár did with GT events.

### Gaps
- Full texts of Bondár 2004, Lomax 2000 and Liu 2022 were not read. Exact teleseismic thresholds and the OBS-relocation uncertainty reporting are unverified.
