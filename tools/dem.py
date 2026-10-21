"""Copernicus GLO-30 DEM tiles (data/dem/*.tif) without imagecodecs: zlib + floating-point predictor by hand."""
import math
import zlib
from pathlib import Path

import numpy as np
import tifffile

DEM_DIR = Path(__file__).resolve().parent.parent / "data" / "dem"


def _undo_fp_predictor(raw, w, h):
    # TIFF predictor 3: per row, byte-plane shuffled (MSB first) then byte-wise horizontal differencing.
    b = np.frombuffer(raw, np.uint8).reshape(h, 4 * w)
    b = np.cumsum(b, axis=1, dtype=np.uint8)
    return b.reshape(h, 4, w).transpose(0, 2, 1).copy().view(">f4").reshape(h, w).astype(np.float32)


def load_tile(lat0, lon0):
    name = f"Copernicus_DSM_COG_10_N{lat0:02d}_00_E{lon0:03d}_00_DEM"
    with tifffile.TiffFile(DEM_DIR / f"{name}.tif") as t:
        p = t.pages[0]
        tw, th = p.tilewidth, p.tilelength
        out = np.zeros(p.shape, np.float32)
        fh = t.filehandle
        cols = math.ceil(p.shape[1] / tw)
        for i, (off, n) in enumerate(zip(p.dataoffsets, p.databytecounts)):
            fh.seek(off)
            tile = _undo_fp_predictor(zlib.decompress(fh.read(n)), tw, th)
            r, c = divmod(i, cols)
            rr, cc = r * th, c * tw
            hh, ww = min(th, p.shape[0] - rr), min(tw, p.shape[1] - cc)
            out[rr:rr + hh, cc:cc + ww] = tile[:hh, :ww]
    return out


class Dem:
    """Elevation lookup over whole-degree tiles; rows run north→south from lat0+1."""

    def __init__(self):
        self.tiles = {}

    def z(self, lat, lon):
        key = (math.floor(lat), math.floor(lon))
        if key not in self.tiles:
            self.tiles[key] = load_tile(*key)
        a = self.tiles[key]
        r = min(int((key[0] + 1 - lat) * a.shape[0]), a.shape[0] - 1)
        c = min(int((lon - key[1]) * a.shape[1]), a.shape[1] - 1)
        return float(a[r, c])
