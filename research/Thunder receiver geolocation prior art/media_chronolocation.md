# Media chronolocation/geolocation from lightning and other natural events (new items beyond research/prior_art_receiver_location.md)

Searched 2026-10-20 (web search + fetch). Several primary pages returned 403 (AMS journals, ResearchGate, Washington Post), and some details come from search-result extracts. These are marked "(snippet-level)".

## 1. Storm/tornado research that syncs or locates videos/photos with lightning detection data

### Takeaway
The peer-reviewed reference is Seimon et al. (BAMS 2016), the formal paper behind the El Reno Survey website already in prior_art. It time-corrects crowdsourced chaser video by matching intervals between visible CG flashes to NLDN strokes (>20 kA). Positions come from scene matching in Google Maps/Street View. Neither timing nor position comes from thunder. Lightning research cameras (Gaisberg, LMA campaigns, UMD Raspberry Pi network) GPS-stamp their frames and correlate them to networks. They are the forward problem with a known clock, not a clock inverse.

### Cited Findings
- **Seimon, A., Allen, J. T., Seimon, T. A., Talbot, S. J., Hoadley, D. K. (2016), "Crowdsourcing the El Reno 2013 Tornado: A New Approach for Collation and Display of Storm Chaser Imagery for Scientific Applications," Bulletin of the American Meteorological Society 97(11).** Authors and year confirmed by the Météo-France library record. The volume/issue comes from the AMS URL path. I did not verify page numbers. — [AMS landing page](https://journals.ametsoc.org/view/journals/bams/97/11/bams-d-15-00174.1.xml); [Météo-France catalogue](https://bibliotheque.meteo.fr/pub/DOC00038223-crowdsourcing-the-reno-2013-tornado-new-approach-f.html)
- The database came from 93 registrants who contributed stills and video with metadata. Lightning flashes were used to time-calibrate the footage. — [Météo-France catalogue](https://bibliotheque.meteo.fr/pub/DOC00038223-crowdsourcing-the-reno-2013-tornado-new-approach-f.html)
- Method (snippet-level, from the AMS page via search): each sequence was "accurately calibrated in time by comparing the time intervals between lightning flash occurrences, particularly cloud-to-ground (CG) flashes, to a reference dataset" (NLDN, ms accuracy). CG frame numbers were logged in a spreadsheet and the pattern matched when a segment held several CGs. Only "CG flashes with peak currents > 20 kA that were recorded on the NLDN" were used. "Video segments were excluded from analysis if they could not be synchronized." — [AMS BAMS 2016](https://journals.ametsoc.org/view/journals/bams/97/11/bams-d-15-00174.1.xml)
- Georeferencing (snippet-level): video scenes were matched to Google Maps and Street View to find chaser lat/lon "at the time of a CG lightning flash". Position is a visual-scene method. — [AMS BAMS 2016](https://journals.ametsoc.org/view/journals/bams/97/11/bams-d-15-00174.1.xml)
- Companion AMS 2016 Annual Meeting talk on the same survey. — [AMS confex Paper290242](https://ams.confex.com/ams/96Annual/webprogram/Paper290242.html)
- **Tipton, KS tornado (May 2019), uncertain.** A Washington Post report (snippet-level, page 403) says a team geolocated chase video with Google Maps, "used lightning flashes to precisely time-synchronize it", and ran photogrammetry on debris for wind speeds. I could not identify the team or a paper. — [Washington Post 2019-06-17](https://www.washingtonpost.com/weather/2019/06/17/this-beastly-tornado-left-behind-captivating-images-an-incredible-set-data/)
- Gaisberg tower (ALDIS, Austria): high-speed video at 1,000 fps, with every image GPS time-stamped for correlation with measured currents and ALDIS locations. — [ALDIS high-speed video](https://www.aldis.at/en/lightning-research/lightning-research-at-gaisberg/high-speed-video-recording/)
- Oklahoma REU study (Ciampa 2012): 10,000 fps video of CG flashes compared frame by frame with the Oklahoma LMA using GPS timing. — [Ciampa final paper](https://reu.caps.ou.edu/reu12/finalpapers/Ciampa-finalpaper.pdf)
- UMD/ESSIC Raspberry Pi lightning camera network: ~90 fps, GPS-stamped frames, 16 cameras as of June 2023. Comparison figures overlay NLDN, ENTLN and GLM. — [ESSIC article](https://essic.umd.edu/building-deploying-tools-to-better-observe-lightning-in-the-washington-d-c-region-beyond/)
- NLDN detects ≥20 kA return strokes with essentially full efficiency and ~80% for 5–20 kA (Nag et al. 2011, cited in an arXiv TGF paper). This supports El Reno's >20 kA cut. — [arXiv 2308.05662](https://arxiv.org/pdf/2308.05662)

### Inferences
- The El Reno matching is an interval-pattern match, which is invariant to the unknown clock offset. It is the optical analogue of lightning-rs's clock-offset scan. The BAMS paper is the citable peer-reviewed precedent for "media clock from a lightning catalogue", where the prior notes had only the website.
- Restricting to >20 kA strokes is a detection-efficiency guard. A thunder method has the same issue on the catalogue side, since missing strokes give unexplained thunder.
- No storm-research item found uses thunder or audio for timing or position. All of them use the optical channel plus GPS or visual georeferencing.

### Gaps
- No VORTEX/TORUS, Wurman or Bluestein paper found that uses NLDN to date frames. The search returned nothing specific, which is not proof of absence.
- El Reno synchronization accuracy (the ≤1 s figure in prior_art) could not be reconfirmed from the BAMS full text (403).
- Tipton team/paper unidentified.

## 2. Lightning-strike forensics (insurance/legal)

### Takeaway
Commercial "lightning verification" (Vaisala STRIKEnet since 1992, now distributed by Weather Fusion and then Cotality) is an address-centred lookup. It lists every NLDN stroke within a radius (typically 10 mi) with time, distance and a per-stroke confidence ellipse. It checks a claimed time and place against the catalogue and does not infer an unknown time or place. The Sago mine disaster investigation (2006) is a notable legal/regulatory case that correlated catalogued strokes with a seismically timed explosion.

### Cited Findings
- STRIKEnet launched in 1992 as the first address-specific weather claim verification tool. Weather Fusion became the exclusive US insurance distributor in 2014. — [Windpower Engineering](https://www.windpowerengineering.com/?p=17328)
- Report method: a search circle, "typically 10 miles in radius, centered on the loss location"; "Each strike is tabulated by time and by distance from the loss location"; "A confidence ellipse is calculated for each strike." The article gives no rules for time windows or detection efficiency. — [Warren Forensics 2013](https://www.warrenforensics.com/2013/11/25/using-historical-weather-data-to-investigate-lightning-strike-claims/)
- A distributor reports that 30% of lightning claims have no strokes detected on the date of loss (snippet-level). — [Windpower Engineering](https://www.windpowerengineering.com/?p=17328)
- Cotality added cloud-to-cloud (Vaisala Xweather) data to Weather Verify STRIKEnet (snippet-level; distributor announcement). — [Cotality](https://www.cotality.com/resources/beyond-the-strike-the-full-lightning-story-doesnt-always-strike-the-ground)
- Vaisala "Strike damage potential" product spotlight. Found but not opened. — [Vaisala PDF](https://Vaisala.com/sites/default/files/documents/WEA-MET-ProductSpotlight-Strike-damage-potential-DID67282EN-A.pdf)
- **Sago Mine (WV, 2 Jan 2006):** MSHA's report concluded lightning was the likely cause. Sandia says three strikes were recorded by national detection networks just before the explosion, which nearby seismographs registered. The MSHA report has a "Vaisala Group and AWS Convergence Technologies" appendix (AA) and a seismic appendix. — [Sandia news release](https://newsreleases.sandia.gov/sandia-research-indicates-that-lightning-was-the-likely-cause-of-sago-mine-explosion/); [MSHA Sago report](https://arlweb.msha.gov/fatals/2006/Sago/sagoreport.asp)
- Timing conflict: Sandia says the bolts struck "simultaneously" with the seismically registered explosion. Mining Monthly, quoting MSHA, says "one to five seconds before". — [Sandia](https://newsreleases.sandia.gov/sandia-research-indicates-that-lightning-was-the-likely-cause-of-sago-mine-explosion/); [Mining Monthly](https://miningmonthly.com/markets/international-coal-news/1279637/msha-releases-sago-report)
- MSHA also has a "Lightning Strike Analysis" appendix for the 2006 Darby mine fatality. — [MSHA Darby Appendix W](https://arlweb.msha.gov/FATALS/2006/Darby/Appendix%20W%20-%20Lightning%20Strike%20Analysis.pdf) (not opened; TLS error on the archive copy of Sago AA)

### Inferences
- Forensic practice is hypothesis testing: given a time and place, is a stroke consistent? It is not search. A thunder method reverses this, so its reports need a likelihood over the hypothesis space and cannot be presented as a lookup.
- Sago is the clearest precedent of catalogue strokes cross-timed against an independently timed physical record (seismic) in an official investigation. It shows that second-level discrepancies get litigated.

### Gaps
- No public Vaisala documentation found on matching rules (time windows, ellipse-based probability of contact). No court ruling found that turns on a STRIKEnet report.
- Earth Networks (ENTLN) forensic product docs not found.
- Contents of Sago Appendix AA (stroke times, ellipses, peak currents) not retrieved.

## 3. Sferic/radio interference in audio used to timestamp recordings

### Takeaway
No source found that timestamps an audio or video recording by matching sferic clicks (AM crackle, EMI pickup) to a lightning catalogue. The physics of audible sferic clicks is documented in patents and popular sources. Network-side matching of sferics uses GPS-tagged receivers.

### Cited Findings
- A lightning-detection patent notes AM radios "are known to suffer from EMI disturbances at a distance up to 30 km or more from a lightning stroke," heard as clicks (snippet-level; patent not opened). — [US 7254484 "Detection of lightning"](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/7254484)
- Los Alamos Sferic Array: GPS UTC-tagged waveforms, matched to NLDN events within 100 µs for accuracy assessment. This is a receiver-network match, not media. — [Smith et al. 2002, JGR](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2001JD000502)
- Single-station sferic location by azimuth and time of arrival. — [Koochak 2020, Radio Science](https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2018rs006627)
- Popular explainer of audible sferics. — [Storm Highway](https://stormhighway.com/sferics.php)

### Inferences
- Sferic clicks would travel at light speed, so a click train matched to catalogue stroke times would give a sub-ms clock independent of position. That is complementary to thunder, which entangles clock and range. It is unexplored in the found literature. Treat this as a hypothesis, not prior art.

### Gaps
- No paper, OSINT guide or repo found for sferic-in-audio chronolocation. Searches were limited to two queries plus patent hits.

## 4. Physics-based geolocation/chronolocation of media from other natural events

### Takeaway
Comparators exist for every channel: crowdsourced video timing of a blast (Beirut, Rigby et al.), many casual videos triangulating a bolide (Chelyabinsk), CCTV footage as a near-fault seismic instrument (Mandalay 2025), and sun/shadow chronolocation as the standard OSINT tool with one quantified automated verifier (IMAGEGUARD). None inverts for an unknown receiver from catalogued events the way lightning-rs does. Each assumes known camera positions and times, or a single known event.

### Cited Findings
- **Rigby, S. E. et al. (2020), "Preliminary yield estimation of the 2020 Beirut explosion using video footage from social media," Shock Waves.** 16 videos with line of sight, blast arrival times at 38 positions, distances from Google Earth. Yield best estimate 0.50 kt TNT, upper limit 1.12 kt. Selection required "synchronised audio and video" and identifiable landmarks. Audio and visual arrival data "broadly follow the same trend". — [Springer s00193-020-00970-z](https://link.springer.com/article/10.1007/s00193-020-00970-z); [White Rose eprint](https://eprints.whiterose.ac.uk/165010)
- Follow-up reanalysis: 0.15–0.7 kt, median ~0.5 kt. — [Springer s00193-021-00992-1](https://link.springer.com/doi/10.1007/s00193-021-00992-1)
- Note: this is distinct from the arXiv 2209.14441 Beirut physics paper already in prior_art.
- **Tianjin 2015:** seismology fixed the two explosion times (15:34:04.68 and 15:34:36.94 UTC, ~32.3 s apart, second ~65 m NW). Infrasound was recorded from 3.5–800 km. No video-timing study found. — [XJTU, "Seismological investigations of two massive explosions in Tianjin"](https://scholar.xjtu.edu.cn/en/publications/seismological-investigations-of-two-massive-explosions-in-tianjin/); [CTBTO SnT2016 abstract](https://conferences.ctbto.org/event/19/contributions/4168/)
- **Chelyabinsk 2013:** Borovička, Spurný, Brown, Wiegert, Kalenda, Clark, Shrbený, "The trajectory, structure and origin of the Chelyabinsk asteroidal impactor," Nature 503:235–237 (2013), doi:10.1038/nature12671. Preliminary trajectory from seven casual videos, calibrated with Google Maps tools, linear path assumed. — [RePEc](https://ideas.repec.org/a/nat/nature/v503y2013i7475d10.1038_nature12671.html); [CBET 3423](https://tamkin3.eps.harvard.edu/cbet/003400/CBET003423.txt)
- Later catalogue of 960 videos (Borovička et al., A&A 2016, snippet-level). Independent dashcam triangulation with distortion compensation reached a few hundred m precision. — [Karmaka catalog note](https://karmaka.de/?p=6510); [arXiv 1403.7438](https://arxiv.org/pdf/1403.7438)
- **Mandalay Mw 7.7 (2025):** Latour, Lebihain, Bhat, Twardzik, Bletery, Hudnut, Passelègue, "Direct Estimation of Earthquake Source Properties from a Single CCTV Camera," arXiv 2505.15461. Measured relative slip per frame. The abstract does not describe how the CCTV clock was handled. A press summary says ~3 m slip in ~1.4 s, peak ~3.5 m/s. — [arXiv 2505.15461](https://arxiv.org/abs/2505.15461); [IRAP/CNRS](https://www.irap.omp.eu/en/2025/11/analysis-of-the-surface-propagation-of-the-2025-mandalay-earthquake-made-possible-by-a-single-surveillance-camera); [Kyoto Univ. 2025-07-24](https://www.kyoto-u.ac.jp/en/research-news/2025-07-24)
- Multi-event CCTV study (Hualien 2018, Mandalay 2025, Kumamoto 2026), arXiv 2608.19656 (snippet-level via aggregator; not verified on arXiv). — [emergentmind](https://www.emergentmind.com/papers/2608.19656)
- EERI/VERT: Exponent synchronized footage from inside a building with the KATNP strong-motion record (Nepal 2015 mainshock). The sync method is unstated. — [EERI Learning From Earthquakes](https://learningfromearthquakes.org/?p=12618)
- **Sun/shadow comparator:** Bellingcat guide (SunCalc, adjust time until shadow length matches, check ±15 min). — [Bellingcat 2021](https://www.bellingcat.com/resources/2021/05/18/unsure-when-a-video-or-photo-was-taken-how-to-tell-by-measuring-the-length-of-shadows/)
- IMAGEGUARD (arXiv 1811.08951) checks time/location claims from object-shadow sun position. It reports 92.5% of true samples recognised and 8.5% error on false ones, on ~200 photos from three countries. — [arXiv 1811.08951](https://arxiv.org/pdf/1811.08951v1)
- Kakar & Sudha: projective-geometry sun position from shadows to verify photo acquisition time (cited in a review). — [arXiv 2402.15448 review](https://arxiv.org/pdf/2402.15448)
- Shadow ambiguity: the same shadow fits a spring and an autumn/winter date. Off-perpendicular views give only a time range. — [Forensic Genealogy](https://forensicgenealogy.info/shadows.html); [cylab.be](https://cylab.be/blog/411/shadow-analysis-1-the-osint-trick-behind-telling-time-from-a-picture)

### Inferences
- Beirut is the closest acoustic analogue in method (video audio arrival vs distance) but runs the other way: known camera positions, unknown source yield. lightning-rs has known sources and an unknown receiver.
- Sun/shadow and ENF are the established comparators with published accuracy and failure modes. IMAGEGUARD's true/false-accept framing is a good template for reporting a thunder method's validation.

### Gaps
- No study found that dates CCTV or phone footage by aligning visible shaking to USGS catalogue origin time plus travel time. The Mandalay paper's clock handling is not verified.
- Chelyabinsk Nature paper's timing (whether video clocks were synced to infrasound or seismic data) not verified.

## 5. Courtroom/forensic standards for audio authentication and timestamping

### Takeaway
SWGDE Best Practices for Digital Audio Authentication (v1.3, 2018) lists ENF as a measurement-based analysis whose uncertainty depends on SNR, distortion, sample rate, bit depth and extraction method. It requires peer-reviewed methods, multiple cross-verified tests and documented decision criteria. Conclusions use a three-level scale (consistent / inconclusive / inconsistent), never absolute certainty, with reports following ASTM E620. A thunder-based time/place estimate would need to fit this frame.

### Cited Findings
- SWGDE, "Best Practices for Digital Audio Authentication," Version 1.3 (20 Sep 2018), 27 pp. Read directly. — [SWGDE PDF](https://www.swgde.org/wp-content/uploads/2023/11/2018-09-20-SWGDE-Best-Practices-for-Digital-Aud.pdf)
  - §4.1: "Use only scientifically valid peer-reviewed methods." "Not all tests will be applicable in each case, nor shall any single analysis be relied upon solely." Decision criteria must be documented.
  - §4.1: analyses are "observed" or "measurement based"; "Measurement uncertainty or measurement error applies to measurement based analyses only."
  - §4.4.8 ENF: "measurement based when used to extract and compare the evidence against a known database. Measurement uncertainty in ENF analysis depends on signal-to-noise ratio, distortion (e.g. clipping, EMF interference), sampling frequency, bit depth, and ENF extraction method."
  - §5.2: results may be "Consistent with an original / Inconclusive / Inconsistent with an original." §5.2.1: "conclusions ... should not be stated in terms of absolutes. Language implying 100% certainty should be avoided."
  - §5.3: follow ASTM E620 (Reporting Opinions of Scientific or Technical Experts). The report must include chain of custody, technical details, and "Data and reasoning relied upon."
  - The document cites Koenig & Lacey, "Forensic Authentication of Digital Audio Recordings," JAES 57(9):662–695 (2009).
- Related SWGDE documents: Best Practices for Forensic Audio v2.5 (2022). The video authentication guidance notes that methods show whether a recording is original or altered, not whether depicted events are authentic. — [SWGDE Forensic Audio v2.5](https://www.swgde.org/wp-content/uploads/2023/11/2022-06-09-SWGDE-Best-Practices-for-Forensic-Audio_v2.5.pdf); [SWGDE 23-V-001 Video Authentication](https://www.swgde.org/documents/published-complete-listing/23-v-001-best-practices-for-digital-video-authentication/)
- ENF literature on forensic limits: performance depends most on test-recording length, then reference length (2024). Noise and interference remain open problems (IEEE Access 2017). Methods "perform poorly with variable ENF trace quality" in forensic speech (2018). Grigoras: spectrogram inspection plus database comparison often suffices for a non-authenticity finding. — [Digital Communications and Networks 2024](https://doaj.org/article/dc6cd16fbf334eb7a0616f4e772303f3); [IEEE Access 2017](https://doaj.org/article/2f367839e8cf45d6a9258d57e410829f); [Proc. Romanian Academy 2018](https://acad.ro/sectii2002/proceedings/doc2018-4/art12.pdf); [Grigoras, IJSLL](https://journal.equinoxpub.com/IJSLL/article/view/9977); [Further considerations, IJSLL](https://journal.equinoxpub.com/IJSLL/article/view/5980)

### Inferences
- To be forensically admissible-shaped, a thunder method would report a measurement uncertainty, such as a calibrated region or time interval with coverage validated on blind cases. lightning-rs's 90% ellipse plus reliability flag maps onto this. It would also need a categorical conclusion (consistent / inconclusive / inconsistent with a claimed time and place) and corroboration by an independent analysis (ENF, shadows, metadata), not sole reliance.
- The forensic use case most likely to fit is verifying a claim ("recorded at X around T"), as STRIKEnet and ENF do, more than open-ended search.

### Gaps
- No court ruling on ENF admissibility (Daubert/Frye or UK) found in these searches. A widely cited "±2 s ENF accuracy" figure came only from an unsourced blog and was excluded.
- AES/ENFSI forensic speech-and-audio guidelines not retrieved.
