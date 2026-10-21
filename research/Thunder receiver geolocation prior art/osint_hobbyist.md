# OSINT / hobbyist prior art: geolocating or timestamping recordings with thunder and lightning catalogues

Searched 2026-10-20. Sources: web search, YouTube page metadata (watch pages and search scraped with a script), GitHub REST search API, `gh search`, Hacker News Algolia API, and Reddit JSON (blocked, so not searched). The baseline list is research/prior_art_receiver_location.md. Items in it are repeated here only where there are new details.

## 1. Public code for thunder + lightning-catalogue geolocation or chronolocation

### Takeaway
I found no public code that matches recorded thunder against a lightning catalogue to locate or time the recorder. The 2026 repos that turned up either synthesise thunder from live strikes or reconstruct lightning channels from microphone arrays.

### Cited Findings
- GitHub repository search for "thunder geolocation", "thunder geolocate", "blitzortung audio", "thunder lightning location audio" and "thunder localization" returned nothing relevant. The only hit was B-bater26/lightning-to-thunder-tracker (last pushed 2026-07-23), a flash-to-bang radius calculator — [GitHub API search](https://api.github.com/search/repositories?q=thunder+geolocation)
- `gh search code` / `gh search repos` returned no results for "blitzortung thunder audio", "thunder geolocate audio", "frost lightning thunder audio" or "thunder envelope lightning strikes cross-correlation". The CLI may have been unauthenticated, so this counts as weak evidence — (no URL; CLI output)
- noah-zim/downstrike, created 2026-08-18 (nine days after colsto's first video), last pushed 2026-10-17, 0 stars. "Real-time lightning storm watcher — live strikes, thunder rings at the speed of sound, distance-modulated thunder." It plays synthetic thunder from Blitzortung WebSocket strikes and does not analyse recorded audio. Nothing I opened shows a link to colsto — [GitHub](https://github.com/noah-zim/downstrike/tree/main), [API metadata](https://api.github.com/repos/noah-zim/downstrike)
- hitenjain22/lightning-project, created 2026-10-06, 0 stars. Simulation-only framework that reconstructs 3D lightning channels from thunder recorded on a microphone array. Method D self-calibrates the array. It uses no catalogue data, does not locate the recorder, and does not mention colsto — [GitHub](https://github.com/hitenjain22/lightning-project)
- Other Blitzortung clients (mrk-its/homeassistant-blitzortung, piRstone/notiblitz, iancharnas/clevelandlightning scraper) fetch strike data or send alerts and do no audio work — [GitHub topic: blitzortung](https://github.com/topics/blitzortung), [clevelandlightning scraper](https://github.com/iancharnas/clevelandlightning/blob/main/scrapeBlitzortungLightningStrikes.py)

### Inferences
- lightning-rs still looks like the only public implementation of the colsto-style inverse (single microphone, catalogue strikes, position grid plus clock scan). colsto's pipeline is described as Python but has not been released.

### Gaps
- GitHub code search needs authentication and returned nothing, so code inside unrelated repos (notebooks, gists) could be missed. I did not search Hugging Face, Kaggle or GitLab.

## 2. Replications, responses and discussion of the colsto method

### Takeaway
Discussion has been thin. The one HN submission got no comments, YouTube search shows no response or replication videos, and I found no Bellingcat, GIJN or newsletter coverage. Reddit could not be searched.

### Cited Findings
- HN story "Locating a livestream using background thunder sounds [video]" (links SnbkIm56TfI), submitted 2026-08-11 by perching_aix, 1 point, no comments — [HN Algolia item 49255716](https://hn.algolia.com/api/v1/items/49255716)
- HN comment by sllabres on 2026-08-19 recommends the @colsto channel in an unrelated thread (parent story 49360545, title not resolved) and links other colsto videos (eY-W9gmwxhg, nzytWZPyuEw, rkmXs_7hELg) — [HN Algolia item 49364782](https://hn.algolia.com/api/v1/items/49364782)
- HN Algolia searches for "thunder geolocate", "geolocate thunder" and "thunder lightning geolocation" turned up no other relevant stories or comments — [HN Algolia API](https://hn.algolia.com/api/v1/search?query=thunder%20sounds%20video)
- YouTube search for "geolocate thunder sounds", "geolocating with lightning" and "thunder triangulation video location" returns only colsto's video as relevant. The rest is ambience content and josemonkey's unrelated "Geolocation Lightning Round" series, so I saw no response videos — [YouTube search](https://www.youtube.com/results?search_query=geolocate+thunder+sounds)
- Only one top comment is embedded on each colsto page, and neither is technical. On SnbkIm56TfI (2026-08-09), @MINPJ: "That 2b2t exploit was finally made real". On 6a7Z0I3LAnE (2026-09-30), @L30WLUV: "Rainbolt but he actually uses rain and bolts" — [video 1](https://www.youtube.com/watch?v=SnbkIm56TfI), [video 2](https://www.youtube.com/watch?v=6a7Z0I3LAnE)
- A general 2026 OSINT techniques guide lists background-noise analysis (spectrograms, audio fingerprinting) and does not mention thunder or lightning catalogues — [ShadowDragon](https://shadowdragon.io/resources/osint-techniques/)

### Inferences
- I found no independent replication or accuracy figure beyond colsto's own claims. Accuracy numbers remain colsto's self-report (2.2 km on the fan clip).

### Gaps
- Reddit's JSON search returned non-JSON (blocked), and the web search engine indexed no matching Reddit threads. r/OSINT, r/geoguessr and r/weather remain unchecked.
- I did not check Bellingcat or OSINT newsletters (Week in OSINT, OSINT Newsletter) issue by issue.

## 3. Benn Jordan's infrasound video (the colsto inspiration)

### Takeaway
colsto credits only "Benn Jordan's video on infrasound". The best candidate is "Infrasound: What You Can't Hear CAN Hurt You" (2025-10-12), which includes a recording of a distant thunderstorm. It records infrasound and does no geolocation or catalogue matching.

### Cited Findings
- colsto's description: "Idea was inspired by Benn Jordan's video on infrasound; would highly recommend if you haven't seen it." No link or title is given — [YouTube SnbkIm56TfI](https://www.youtube.com/watch?v=SnbkIm56TfI)
- Benn Jordan, "Infrasound: What You Can't Hear CAN Hurt You", uploaded 2025-10-12, about 894k views. Description: "Our world is filled with inaudible sounds that a lot of research suggests is harmful to our health. I wanted to devise a way to hear it." — [YouTube UTvr8L5v8u8](https://www.youtube.com/watch?v=UTvr8L5v8u8)
- In that video Jordan records infrasound at a remote site south of Badlands NP, South Dakota. At about 11:38 he plays a distant thunderstorm ("made a symphony of various rumbles") and says infrasound can help detect tornadoes. Neither source mentions locating sources or matching to lightning data. This is according to a critic's summary, not a neutral source — [Andy Masley, "Contra Benn Jordan"](https://blog.andymasley.com/p/contra-benn-jordan-data-center-and)
- Second candidate: Benn Jordan, "Datacenters Behaving Like Acoustic Weapons", uploaded 2026-02-18, about 1.84M views. Chapters include Infrasound, xAI/Colossus, MARA Crypto Mine, Permian Basin and Recording Techniques. The description says nothing about thunder — [YouTube _bP80DEAbuo](https://www.youtube.com/watch?v=_bP80DEAbuo)

### Inferences
- The inspiration was most likely hearing thunder as a recordable, distinctive signal in the 2025 video, not a method. Jordan provides no inversion technique.

### Gaps
- Uncertain which of the two videos colsto meant. YouTube's caption endpoint returned empty transcripts (it needs a PO token), so I could not confirm the thunder segment directly.

## 4. colsto: code, write-ups, follow-ups, transcript details

### Takeaway
I found no code, write-up or third video. The descriptions add only small details beyond the baseline file. I could not get transcripts.

### Cited Findings
- Video 1 upload timestamp is 2026-08-09T07:47:21-07:00 with about 416k views. The description says commercial data (Xweather, Vaisala GLD360, Earth Networks ENTLN) could improve per-strike accuracy. Tests: blind validation on a known location, a whole-world search with known date and unknown time, and a known region with about 3 weeks of time uncertainty — [YouTube SnbkIm56TfI](https://www.youtube.com/watch?v=SnbkIm56TfI)
- Video 2 upload timestamp is 2026-09-30T14:55:50-07:00 with about 88k views. Uses MET Norway public API strikes ("~200m vs ~3km" Blitzortung) and selective weighting that favours strikes outside the main cluster. Error was 2.2 km, attributed to strikes clustered away from the listener. "With strikes coming from multiple directions, ~500m accuracy should be feasible." The video is sponsored by TryHackMe — [YouTube 6a7Z0I3LAnE](https://www.youtube.com/watch?v=6a7Z0I3LAnE)
- Channel self-description: OSINT and geolocation case studies, with a business contact at Lighthouseagents.com. Other videos are non-thunder OSINT, for example "I used OSINT to find the real One Piece ocean drop location" (2026-06-13, about 611k views) — [channel](https://www.youtube.com/@colsto), [eY-W9gmwxhg](https://www.youtube.com/watch?v=eY-W9gmwxhg)
- Both thunder videos have only English auto-generated (ASR) captions. The timedtext fetch came back empty — (YouTube watch-page captionTracks; no separate URL)

### Inferences
- Neither description mentions an uncertainty region, likelihood or reliability flag. The baseline's tentative "no UQ" point still stands but is unverified without the transcript.

### Gaps
- Transcript content (exact error figures, grid resolution, how the clock offset was handled). A transcript service such as youtubetranscript.com or downsub, or yt-dlp with cookies, is the next step.

## 5. Lightning in OSINT chronolocation of conflict/news videos

### Takeaway
I found no Bellingcat, Geoconfirmed, Forensic Architecture or NYT Visual Investigations case that uses lightning catalogues to chronolocate footage. Published chronolocation relies on shadows and sun position, and El Reno (in the baseline) remains the clearest flash-to-catalogue timing case.

### Cited Findings
- Bellingcat's Minab school strike investigation (2026-03-27) timed the videos with solar and shadow analysis, not weather or lightning — [Bellingcat](https://www.bellingcat.com/news/2026/03/27/new-videos-reveal-more-details-about-iran-girls-school-strike/)
- Bellingcat's advanced video verification guide covers upload-time tools and thumbnails, with no lightning method — [Bellingcat 2017 guide](https://www.bellingcat.com/resources/how-tos/2017/06/30/advanced-guide-verifying-video-content/)
- An OSINT toolkit suggests checking identifiable background sounds against claimed time and place using reference data such as flight tracking. This is the same principle, but thunder is not named — [NezrKaan/osint-toolkit](https://github.com/NezrKaan/osint-toolkit)
- Practical obstacle: Blitzortung raw data with coordinates and times is available only to contributors, historical LightningMaps data shows only maps, and the API needs credentials. This is a barrier for after-the-fact OSINT use of Blitzortung, which pushes people to MET Norway Frost (as colsto did) — [Blitzortung forum](https://forum.blitzortung.org/portal.php?page=4), [limaps.org](https://www.limaps.org/live-data.html)

### Gaps
- I did not manually search Geoconfirmed, FA or the NYT archives. "Not found" means search engines surfaced nothing.

## 6. Hobbyist tools linking heard thunder to strike data

### Takeaway
Hobbyist tools go the forward direction (catalogue strike → predicted thunder arrival at a known listener), through LightningMaps sound rings, Home Assistant and downstrike. Users informally compare heard thunder with them, but nobody inverts. The closest conceptual precursor is a Minecraft exploit that triangulates player positions from global thunder-sound packets.

### Cited Findings
- LightningMaps.org: each new strike spawns a ring expanding at about the speed of sound, and thunder should arrive when the ring reaches you. It is modelled, not measured, and wind, terrain and channel geometry shift actual arrival times. Article dated 2026-08-04 by Jan Bielik — [webiano.digital](https://webiano.digital/lightningmaps-org-lets-you-watch-thunder-arrive/)
- Blitzortung forum users report hearing thunder just before the ring reaches them. One estimates distance by starting a count at the map's displayed delay, and notes that "perhaps half the time" there is thunder with no Blitzortung strike, an anecdotal detection-efficiency gap — [forum thread 2014-08-26](https://forum.blitzortung.org/showthread.php?pid=10747)
- The Home Assistant community has a "Blitzortung, showing thunder on map" thread that forward-models thunder for a home location — [HA community](https://community.home-assistant.io/t/blitzortung-showing-thunder-on-map/214560)
- Minecraft thunder leak: servers send the thunder sound packet with true coordinates to players within 160,000 blocks, and because lightning occurs only near loaded chunks, players can be located to about 500 blocks. Patch plugins randomise far coordinates, and server operators disable thunder as a workaround. colsto's commenter calls this "That 2b2t exploit" — [Confined Thunder plugin](https://dev.curseforge.com/projects/localized-thunder), [Minecraft feedback post](https://feedback.minecraft.net/hc/en-us/community/posts/360075979031-add-option-to-disable-global-sounds-on-servers)
- FitMC, "2b2t's History of Lightning" (about 7 years old, about 3.4M views) appears in YouTube search for "thunder triangulation video location". I took the title from search results and did not watch it, so it is unverified that it covers the exploit — [YouTube T2H527m60bQ](https://www.youtube.com/watch?v=T2H527m60bQ)

### Inferences
- The forum's "half the time no strike" remark matches this repo's audible-strike-count limit. Catalogue detection efficiency for the strokes that are heard is a real constraint, not just location error.

### Gaps
- I did not search AS3935 hobby-sensor projects or storm-chaser apps (RadarScope, My Lightning Tracker) specifically for audio features. None surfaced in general searches.
