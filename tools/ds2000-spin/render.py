"""Regenerates static/images/DS2000/logo/ds2000-spin.webp from the two sources
next to this script. Needs Chrome and Pillow: python render.py [frames] [ms].

Pre-renders the spinning DS2000 logo to an animated WebP (transparent).

Frames come from headless Chrome (real CSS 3D, 64 dense layers so the slab
looks solid); the cubic ease-in-out is baked into the angle per frame.
"""
import os
import subprocess
import sys
from PIL import Image

S = os.path.dirname(os.path.abspath(__file__))
R = "file:///" + S.replace(os.sep, "/") + "/"
CHROME = r"C:\Program Files\Google\Chrome\Application\chrome.exe"
N = 64
FRAMES = int(sys.argv[1]) if len(sys.argv) > 1 else 96
STEP_MS = int(sys.argv[2]) if len(sys.argv) > 2 else 62

imgs = "".join(
    f'<img src="{R}ds2000-logo-3d-edge.webp" style="transform:translateZ({(i - N / 2) * 0.6:.2f}px)">'
    for i in range(N)
)
imgs += f'<img src="{R}ds2000-logo-3d.webp" style="transform:translateZ({N / 2 * 0.6:.2f}px)">'
html = f"""<!doctype html><meta charset=utf-8><style>
html,body{{margin:0;background:transparent;overflow:hidden}}
#s{{width:440px;height:440px;perspective:1100px}}
#r{{position:relative;width:440px;height:440px;transform-style:preserve-3d}}
img{{position:absolute;left:30px;top:25px;width:380px;height:389px}}
</style><div id=s><div id=r>{imgs}</div></div>
<script>const a=new URLSearchParams(location.search).get('a')||0;
r.style.transform=`rotateX(-10deg) rotateY(${{a}}deg)`</script>"""
page = os.path.join(os.environ.get("TEMP", S), "ds2000-spin.html")
with open(page, "w", encoding="utf-8") as f:
    f.write(html)

FR = os.path.join(os.environ.get("TEMP", S), "ds2000-spin-frames")
os.makedirs(FR, exist_ok=True)


def ease(t):  # cubic ease-in-out
    return 4 * t**3 if t < 0.5 else 1 - (-2 * t + 2) ** 3 / 2


frames = []
for k in range(FRAMES):
    ang = 360 * ease(k / FRAMES)
    out = os.path.join(FR, f"f{k:03d}.png")
    subprocess.run(
        [CHROME, "--headless=new", "--disable-gpu", "--hide-scrollbars",
         "--default-background-color=00000000", "--window-size=440,440",
         "--virtual-time-budget=1500", f"--screenshot={out}",
         f"file:///{page.replace(os.sep, '/')}?a={ang:.2f}"],
        check=True, capture_output=True,
    )
    frames.append(Image.open(out).convert("RGBA"))

dest = os.path.join(S, "..", "..", "static", "images", "DS2000", "logo", "ds2000-spin.webp")
frames[0].save(dest, save_all=True, append_images=frames[1:], duration=STEP_MS,
               loop=0, lossless=False, quality=80, method=6, minimize_size=True)
print(os.path.getsize(dest), "bytes", len(frames), "frames")
