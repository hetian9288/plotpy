// The code in `set_equal_axes` is based on:
// https://stackoverflow.com/questions/13685386/matplotlib-equal-unit-length-with-equal-aspect-ratio-z-axis-is-not-equal-to
//
// It needs Matplotlib version at least 3.3.0 (Jul 16, 2020)
// https://github.com/matplotlib/matplotlib/blob/f6e0ee49c598f59c6e6cf4eefe473e4dc634a58a/doc/users/prev_whats_new/whats_new_3.3.0.rst

/// Commands to be added at the beginning of the Python script
///
/// The python functions are:
///
/// * `add_to_ea` -- Adds an entity to the EXTRA_ARTISTS list to prevent them being ignored
///    when Matplotlib decides to calculate the bounding boxes. The Legend is an example of entity that could
///    be ignored during by the savefig command (this is issue is prevented here).
/// * `maybe_create_ax3d` -- If AX3D is None, allocates a new mplot3d (Matplotlib's 3D plotting capability)
/// * `data_to_axis` -- Transforms data limits to axis limits
/// * `axis_to_data` -- Transforms axis limits to data limits
/// * `set_equal_axes` -- Configures the aspect of axes with a same scaling from data to plot units for x, y and z.
///   For example a circle will show as a circle in the screen and not an ellipse. This function also handles
///   the 3D case which is a little tricky with Matplotlib. In this case (3D), the version of Matplotlib
///   must be greater than 3.3.0.
/// * TODO: find a way to pass down the option `proj_type = 'ortho'` to AX3D
pub const PYTHON_HEADER: &str = "### file generated by plotpy
import time
import numpy as np
import matplotlib.pyplot as plt
import matplotlib.ticker as tck
import matplotlib.patches as pat
import matplotlib.path as pth
import matplotlib.patheffects as pff
import matplotlib.lines as lns
import matplotlib.transforms as tra
import mpl_toolkits.mplot3d as m3d
NaN = np.NaN
EXTRA_ARTISTS = []
def add_to_ea(obj):
    if obj!=None: EXTRA_ARTISTS.append(obj)
COLORMAPS = [plt.cm.bwr, plt.cm.RdBu, plt.cm.hsv, plt.cm.jet, plt.cm.terrain, plt.cm.pink, plt.cm.Greys]
def get_colormap(idx): return COLORMAPS[idx % len(COLORMAPS)]
AX3D = None
def maybe_create_ax3d():
    global AX3D
    if AX3D == None:
        AX3D = plt.gcf().add_subplot(111, projection='3d')
        AX3D.set_xlabel('x')
        AX3D.set_ylabel('y')
        AX3D.set_zlabel('z')
        add_to_ea(AX3D)
def data_to_axis(coords):
    plt.axis() # must call this first
    return plt.gca().transLimits.transform(coords)
def axis_to_data(coords):
    plt.axis() # must call this first
    return plt.gca().transLimits.inverted().transform(coords)
def set_equal_axes():
    ax = plt.gca()
    if AX3D == None:
        ax.axes.set_aspect('equal')
        return
    try:
        ax.set_box_aspect([1,1,1])
        limits = np.array([ax.get_xlim3d(), ax.get_ylim3d(), ax.get_zlim3d()])
        origin = np.mean(limits, axis=1)
        radius = 0.5 * np.max(np.abs(limits[:, 1] - limits[:, 0]))
        x, y, z = origin
        ax.set_xlim3d([x - radius, x + radius])
        ax.set_ylim3d([y - radius, y + radius])
        ax.set_zlim3d([z - radius, z + radius])
    except:
        import matplotlib
        print('VERSION of MATPLOTLIB = {}'.format(matplotlib.__version__))
        print('ERROR: set_box_aspect is missing in this version of Matplotlib')

def loop_wait():
    while True:
        time.sleep(1)

        
";

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::PYTHON_HEADER;

    #[test]
    fn constants_are_correct() {
        assert_eq!(PYTHON_HEADER.len(), 1768);
    }
}
