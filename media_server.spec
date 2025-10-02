# media_server.spec
# -*- mode: python ; coding: utf-8 -*-

block_cipher = None

a = Analysis(
    ['media_server.py'],
    pathex=[],
    binaries=[],
    datas=[('controller_black_app_icon.icns', '.')],  # only simple 2-tuples here
    hiddenimports=[],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    win_no_prefer_redirects=False,
    win_private_assemblies=False,
    cipher=block_cipher,
    noarchive=False,
)

# ✅ extend a.datas here
a.datas += Tree('templates', prefix='templates')
a.datas += Tree('static', prefix='static')

pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='media_server',
    console=False,
    icon='controller_black_app_icon.icns'
)

coll = COLLECT(
    exe,
    a.binaries,
    a.zipfiles,
    a.datas,
    strip=False,
    upx=True,
    name='media_server'
)

app = BUNDLE(
    coll,
    name='media_server.app',
    icon='controller_black_app_icon.icns',
    bundle_identifier=None
)
