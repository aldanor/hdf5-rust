import argparse
import os
import subprocess
import urllib.request
import winreg


def download(url, filename):
    print('\nDownloading:', url, flush=True)
    urllib.request.urlretrieve(url, filename)


def run_command(*args, **kwargs):
    print('\n>', *args, flush=True)
    subprocess.check_call(list(args), **kwargs)


def install_rust():
    channel = os.getenv('CHANNEL')
    assert channel
    print(f'Installing Rust ({channel})...')
    msi = f'rust-{channel}-x86_64-pc-windows-msvc.msi'
    url = f'https://static.rust-lang.org/dist/{msi}'
    download(url, msi)
    run_command('msiexec', '/i', msi, r'INSTALLDIR=C:\Rust', '/qn')
    run_command(r'C:\Rust\bin\rustc', '-vV')
    run_command(r'C:\Rust\bin\cargo', '-vV')


def install_hdf5():
    source = os.getenv('H5_SOURCE')
    assert source in ('msi', 'conda')
    version = os.getenv('H5_VERSION')
    assert version
    print(f'Installing HDF5 ({version}, source={source})...')
    if source == 'conda':
        run_command('conda', 'config', '--set', 'always_yes', 'yes')
        run_command('conda', 'config', '--set', 'changeps1', 'no')
        run_command('conda', 'create', '-y', '-n', 'testenv', f'hdf5=={version}')
    else:
        sources = {
            '1.8.21': ('hdf5-1.8.21-Std-win7_64-vs14.zip',
                       r'hdf\HDF5-1.8.21-win64.msi'),
            '1.10.0': ('windows/extra/hdf5-1.10.0-win64-VS2015-shared.zip',
                       r'hdf5\HDF5-1.10.0-win64.msi'),
        }
        path, msi = sources[version]
        family = version.rsplit('.', 1)[0]
        url = (
            'https://support.hdfgroup.org/ftp/HDF5/prev-releases'
            f'/hdf5-{family}/hdf5-{version}/bin/{path}'
        )
        download(url, 'hdf5.zip')
        run_command('7z', 'x', 'hdf5.zip', '-y')
        run_command('msiexec', '/i', msi, '/qn')


def run_tests():
    print('Running tests...')
    source = os.getenv('H5_SOURCE')
    env = os.environ.copy()
    if source == 'conda':
        conda_root = env['CONDA']
        envdir = rf'{conda_root}\envs\testenv'
        print(f'\nSetting HDF5_DIR to {envdir}...')
        env['HDF5_DIR'] = envdir
        for path in ('Scripts', r'Library\bin', ''):
            envpath = fr'{envdir}\{path}'
            print(f'\nPrepending {envpath} to %PATH%...')
            env['PATH'] = envpath + ';' + env['PATH']
    else:
        prog_key = 'SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall'
        root = winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, prog_key)
        for i in range(winreg.QueryInfoKey(root)[0]):
            sub = winreg.OpenKey(root, winreg.EnumKey(root, i))
            prog = dict(
                tuple(winreg.EnumValue(sub, j)[:2])
                for j in range(winreg.QueryInfoKey(sub)[1])
            )
            if prog.get('DisplayName') != 'HDF5':
                continue
            install_location = prog['InstallLocation']
            bin_dir = os.path.join(install_location, 'bin')
            print(f'\nPrepending {bin_dir} to %PATH%...')
            env['PATH'] = bin_dir + ';' + env['PATH']
    if os.getenv('PIN_VERSION'):
        env['HDF5_VERSION'] = env['H5_VERSION']
        print('Pinning HDF5 version to', env['HDF5_VERSION'])
    run_command('cargo', 'build', '-vv', env=env)
    run_command('cargo', 'test', '-v', '--all', env=env)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('action', choices=[
        'install-rust', 'install-hdf5', 'run-tests',
    ])
    args = parser.parse_args()
    if args.action == 'install-rust':
        install_rust()
    elif args.action == 'install-hdf5':
        install_hdf5()
    elif args.action == 'run-tests':
        run_tests()


if __name__ == '__main__':
    main()
