# Prior art: locating/timing a RECEIVER from thunder (or other catalogued sources)

Searched 2026-10-13: web, Semantic Scholar API, arXiv API, Google Scholar, GitHub API, YouTube metadata.

## Direct precedent (same problem)

1. **colsto (YouTube, OSINT channel), "Can you geolocate a video using background thunder sounds?"** Uploaded 2026-08-09, 14:48 long, ~416k views. https://www.youtube.com/watch?v=SnbkIm56TfI
   Method, from the description: logged the Blitzortung/lightningmaps.org socket feed for about 3 weeks. For each candidate location it models an acoustic curve (slant distance, inverse-square attenuation) and compares it with the audio's 20–250 Hz RMS envelope by cosine similarity, using FFT cross-correlation over all time offsets. Validated blind on a known location, then tested on (1) whole-world search with known date and unknown time, and (2) known region with ~3-week time uncertainty. Says the idea came from Benn Jordan's infrasound video. **Same core method as this repo: grid over position, scan clock offset, compare envelopes.**
2. **colsto, "I located a fan's video using thunder sounds."** Uploaded 2026-09-30, 11:11 long, ~87k views. https://www.youtube.com/watch?v=6a7Z0I3LAnE
   Switched to **MET Norway (Frost) public API** strikes ("~200 m vs ~3 km" for Blitzortung) and added selective weighting of strikes outside the main cluster. Result: 2.2 km error, blamed on strikes clustered on one side of the listener ("~500 m feasible" with strikes from several directions). **Same catalog as this repo, and it hit the same failure mode as memory/sim-suite-findings: geometry and the audible-strike count limit locatability.**
   Uncertain: no code, paper or repo found for either video. Details come only from the descriptions. Transcripts were not reviewed.

## Adjacent: timing (chronolocation) from a lightning catalog

3. **El Reno Survey (2013 El Reno tornado).** https://el-reno-survey.net/update-24-nov-13
   Matched *visible* flashes in video to NLDN CG strikes to fix camera clock offsets to ≤1 s. Needs several flashes per clip. Camera positions came from GPS and road analysis, not thunder. This is a clock-only inverse that uses the optical channel.
4. **Raspberry Shake forum, infrasound signal vs LightningMaps strike times.** https://community.raspberryshake.org/t/unidentified-boom-detected-near-oberon-nsw-australia/4165
   Anecdotal. Matches an infrasound/seismic arrival to catalogued strike times to explain a signal. No inversion.

## Adjacent: receiver self-location / clock from catalogued sources

5. **US9960901B2, "Clock synchronization using sferic signals"** (US Navy, Gorgen & Lemay, filed 2016). https://patents.google.com/patent/US9960901
   Uses lightning sferics as signals of opportunity to estimate receiver clock errors and antenna positions. It uses TDOA across networked receivers and **no catalog**, so the EM analogue is weak.
6. **US4914444A, "System and method for synchronizing lightning detectors."** https://patents.google.com/patent/US4914444
   Correlates common lightning events to estimate detector clock offsets.
7. **Seismic clock-error correction from catalog teleseisms:** Sukhovich et al., BSSA 2020 (hydrophone drift from teleseismic P), https://hal.univ-brest.fr/hal-03040836/document. Marapi volcano network (teleseismic + ambient noise), https://earthobservatory.sg/research/publications/clock-error-detection-for-decade-long-seismic-monitoring-network-in-marapi-volcano-indonesia. USGS timing-accuracy study (repeating earthquakes; errors from 10s of ms to over 10 s), https://pubs.usgs.gov/publication/70226473.
   This is the standard "known sources → receiver clock" inverse. The receiver position is known.
8. **Seismic station mislocation from catalog residuals:** Haase et al. 1995, GPS resurvey of SCSN (12% of stations off by more than 500 m, found via systematic travel-time residuals). https://resolver.caltech.edu/CaltechAUTHORS:20130222-080301036
   Receiver position error inferred from catalogued sources. The closest geophysical analogue, though it uses a dense network.
9. **DAS channel geolocation from a geotracked vehicle** (SRL 2023, Long Valley). https://par.nsf.gov/servlets/purl/10405143 (URL from the search result. I did not open the PDF to confirm the title.)
   Locates receivers (fiber channels) from a source with known position and time. EGU26-17223 (NORFOX) suggests lightning as a timing/spatial reference for DAS. https://meetingorganizer.copernicus.org/EGU26/EGU26-17223.html

## Adjacent: microphone self-localization with unknown sources (no catalog)

10. El Badawy, Larsson, Pollefeys, Dokmanić, "Localizing Unsynchronized Sensors with Unknown Sources" (2021). https://arxiv.org/abs/2102.03565
11. Wang, Hon, Reiss, Cavallaro, "Self-localization of ad-hoc arrays using time difference of arrivals" (2015). https://qmro.qmul.ac.uk/xmlui/bitstream/123456789/17626/1/Reiss%20Self-localization%20of%20ad-hoc%202015%20Accepted.pdf
12. Woźniak, Kowalczyk, Cobos, "Passive joint localization and synchronization of distributed microphone arrays" (ICASSP 2020). https://rc.signalprocessingsociety.org/conferences/icassp-2020/spsicassp20vid1897
    These need multiple receivers and estimate relative geometry. This repo has a single receiver and absolute source positions/times from a catalog.

## Adjacent: forensic geolocation/chronolocation of recordings by other cues

13. **ENF (mains hum):** Garg et al., ICASSP 2013, https://www.mast.umd.edu/files/paper_ENF/conf_paper6_ICASSP13.pdf. Intra-grid feasibility 2021, https://arxiv.org/abs/2105.00668. InterGridNet 2025, https://arxiv.org/abs/2502.10011.
    The established forensic "where/when from audio" route. It works at grid scale, and only coarsely within a grid.
14. **Flash-to-bang ranging of single recordings:** davidnewschool/sound-delay, https://github.com/davidnewschool/sound-delay. Beirut-explosion physics paper, https://arxiv.org/abs/2209.14441.
    Gives camera-to-source distance from one known event. A single-event range ring, with no catalog search.
15. **Audio-visual video geolocation against a database** (La Salle-URL 2015; IARPA LocUS). https://www.sciencedaily.com/releases/2015/02/150216064741.htm, https://www.iarpa.gov/research-programs/locus
    Ambient-sound fingerprinting, not physics-based.

## Checked, nothing relevant
- Semantic Scholar "thunder recording location estimation lightning detection": the top 20 of 487 hits all locate the lightning, not the receiver.
- arXiv API "thunder AND localization": no lightning papers.
- Google Scholar: no hit estimates the recorder position from thunder.
- GitHub "thunder geolocation": no matching repo.
- Bellingcat/GIJN: chronolocation guides cover shadows and weather, with no thunder method found.
- Lu et al., Sensors 2023 (crowd-sourced smartphone thunder localization), https://pmc.ncbi.nlm.nih.gov/articles/PMC10181301/: phone positions are known, and it locates the thunder.

## Verdict on novelty
**The core idea is not novel.** colsto published the same inversion publicly in Aug 2026 (Blitzortung) and again in Sep 2026 on the same MET Norway catalog: envelope match over a position grid plus a clock-offset scan, on single-mic audio. In the second video it reports ~2.2 km error and a geometry-limited failure like this repo's. I found **no peer-reviewed paper, preprint or public code** for thunder-plus-catalog receiver localization. Peer-reviewed analogues exist only in seismology (station clock and position from catalogued events) and as sferic patents. What could still be new and publishable is method rigor rather than the concept: a calibrated likelihood with uncertainty regions and a reliability flag, channel onset height, atmosphere nuisance terms, a simulation suite for locatability, and a blind holdout. colsto's descriptions mention no uncertainty quantification. That last point is uncertain because transcripts and code were not reviewed.
