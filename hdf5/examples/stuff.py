from h5py import File
import numpy as np

# http://docs.h5py.org/en/stable/high/dims.html

f = File('dims_1d.h5', 'w')

f['x1'] = [1, 2]
f['x1'].make_scale('x1 name')
f['data'] = np.ones((2,), 'f')
f['data'].dims[0].attach_scale(f['x1'])
