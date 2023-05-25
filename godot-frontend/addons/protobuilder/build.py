#!/usr/bin/env python3

import subprocess
import os
import logging
import errno

PROTO_DIR = os.path.join(os.getcwd(), "protos")
GENDIR = os.path.join(os.getcwd(), 'gen')
# Godot trashes the env, so we need to manually specify path.
PROTOC_PATH = '/opt/homebrew/bin/protoc'

logging.basicConfig(
    filename='.godot/editor/protobuf_gen.log', encoding='utf-8',
    level=logging.DEBUG, format='%(asctime)s %(message)s')
logging.debug('Running')
logging.debug('Env:' + str(os.environ))

def log_subprocess_output(pipe):
    for line in iter(pipe.readline, b''):
        logging.info('stdout: %r', line)
        print(line)

def mkdir_p(path):
    try:
        os.makedirs(path)
    except OSError as exc:
        if exc.errno == errno.EEXIST and os.path.isdir(path):
            pass
        else:
            raise

for root, dirs, files in os.walk(PROTO_DIR):
    # Copy dir structure
    postfix = root[len(os.getcwd())+len(os.pathsep):]
    outdir = os.path.join(GENDIR, postfix)
    mkdir_p(outdir)
    print(outdir)

    for f in files:
        if f.endswith('.proto'):
            f = os.path.join(root, f)
            args = [PROTOC_PATH, '--proto_path='+PROTO_DIR, '--csharp_out=' + outdir, f]
            logging.info('Calling: ' + str(args))
            process = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            with process.stdout:
                log_subprocess_output(process.stdout)