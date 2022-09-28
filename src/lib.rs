use numpy::PyArray;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use rand::{thread_rng, Rng};

extern crate las;
extern crate startin;

use las::point::Classification;
use las::Read;

/// A Delaunay triangulator where the input are 2.5D points,
/// the DT is computed in 2D but the elevation of the vertices are kept.
/// This is used mostly for the modelling of terrains.
/// This is the Python bindings of Rust's startin:
/// (https://github.com/hugoledoux/startin)
#[pymodule]
fn startinpy(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<DT>()?;
    Ok(())
}

#[pyclass(unsendable)]
/// A Delaunay triangulation (DT), containing vertices+triangles
pub struct DT {
    t: startin::Triangulation,
}

#[pymethods]
impl DT {
    /// Constructor for a DT (returns an empty DT)
    #[new]
    fn new() -> Self {
        let tmp = startin::Triangulation::new();
        DT { t: tmp }
    }

    /// Get the points [x, y, z] of all vertices in the DT.
    /// This includes the infinite vertex (vertex at position 0), which is not part of the DT.
    /// It has dummy coordinates and no triangles refer to it.
    ///
    /// >>> pts = dt.points
    /// >>> print(pts.shape)
    /// (102, 3) #-- this is a numpy array
    /// >>> for p in pts:
    /// >>>     print(p[0], p[1], p[2])
    /// ...
    /// >>> dt.points[27]
    /// [101.122 72.293 11.223]
    /// >>> dt.points[0]
    /// [-99999.99999 -99999.99999 -99999.99999]
    #[getter]
    fn points<'py>(&self, py: Python<'py>) -> PyResult<&'py PyArray<f64, numpy::Ix2>> {
        let vs = self.t.all_vertices();
        Ok(PyArray::from_vec2(py, &vs).unwrap())
    }

    /// Get the triangles in the DT.
    ///
    /// >>> trs = dt.triangles
    /// >>> print(trs.shape)
    /// (224, 3) #-- this is a numpy array
    /// >>> one_triangle = trs[22]
    /// >>> first_vertex = one_triangle[0]
    /// >>> print("x-coordinate of first vertex: ", dt.points[first_vertex])
    /// x-coordinate of first vertex: [25.98 35.12 4.78]
    #[getter]
    fn triangles<'py>(&self, py: Python<'py>) -> PyResult<&'py PyArray<usize, numpy::Ix2>> {
        let mut trs: Vec<Vec<usize>> = Vec::with_capacity(self.t.number_of_triangles());
        for each in self.t.all_triangles() {
            let mut tr = Vec::with_capacity(3);
            tr.push(each.v[0]);
            tr.push(each.v[1]);
            tr.push(each.v[2]);
            trs.push(tr);
        }
        Ok(PyArray::from_vec2(py, &trs).unwrap())
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("{}", self.t))
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("{}", self.t))
    }

    /// Insert one new point in the DT.
    /// If there is a point at the same location (based on :func:`startinpy.DT.snap_tolerance`),
    /// then the point is not inserted and the index of the already existing vertex is returned.
    ///
    /// :param x: x-coordinate of point to insert
    /// :param y: y-coordinate of point to insert
    /// :param z: z-coordinate of point to insert
    /// :return: index of the vertex in the DT   
    ///
    /// >>> dt.insert_one_pt(3.2, 1.1, 17.0)
    /// 5
    /// (the vertex index in the DT is 5)
    #[pyo3(text_signature = "($self, x, y, z)")]
    fn insert_one_pt(&mut self, x: f64, y: f64, z: f64) -> PyResult<usize> {
        let re = self.t.insert_one_pt(x, y, z);
        match re {
            Ok(x) => return Ok(x),
            Err(x) => return Ok(x),
        };
    }

    /// Remove/delete the vertex vi (an index) from the DT, and update the DT for the Delaunay criterion.
    ///
    /// :param vi: index of vertex to delete
    /// :return: (Exception is thrown if *vi* is invalid)
    ///
    /// >>> try:
    /// >>>     t.remove(45)
    /// >>> except Exception as e:
    /// >>>     print(e)
    #[pyo3(text_signature = "($self, vi)")]
    fn remove(&mut self, vi: usize) -> PyResult<()> {
        let re = self.t.remove(vi);
        match re {
            Ok(_x) => return Ok(()),
            Err(why) => match why {
                startin::StartinError::VertexInfinite => {
                    return Err(PyErr::new::<exceptions::PyIndexError, _>(
                        "Invalid index, cannot remove infinite vertex.",
                    ));
                }
                _ => {
                    return Err(PyErr::new::<exceptions::PyIndexError, _>(
                        "Invalid index, vertex doesn't exist.",
                    ));
                }
            },
        };
    }

    /// Insert each point in the array of points (a 2D array) by calling insert_one_pt() for each.
    /// Different insertion strategies can be used: "AsIs" (*default*: inserts points in the order
    /// given) or "BBox" (inserts first the BBox of the points, which speeds up the construction,
    /// works especially good for rasters).
    ///
    /// :param pts: an array of points (which is an array)
    /// :param optional insertionstrategy:  "AsIs" (*default*) or "BBox"
    /// :return: (nothing)
    ///
    /// >>> pts = []
    /// >>> pts.append([1.0, 1.0, 11.11])
    /// >>> pts.append([1.0, 2.3, 22.22])
    /// >>> pts.append([12.3, 21.0, 4.52])
    /// >>> ...
    /// >>> dt = startinpy.DT()
    /// >>> dt.insert(pts)
    /// OR
    /// >>> dt.insert(pts, insertionstrategy="BBox")
    #[pyo3(text_signature = "($self, pts, *, insertionstrategy='AsIs')")]
    #[args(path, insertionstrategy = "\"AsIs\"")]
    fn insert(&mut self, pts: Vec<[f64; 3]>, insertionstrategy: &str) -> PyResult<()> {
        match insertionstrategy {
            "AsIs" => self.t.insert(&pts, startin::InsertionStrategy::AsIs),
            "BBox" => self.t.insert(&pts, startin::InsertionStrategy::BBox),
            _ => {
                let s = format!(
                    "'{}' is an unknown insertion strategy for insert()",
                    insertionstrategy
                );
                return Err(PyErr::new::<exceptions::PyTypeError, _>(s));
            }
        }
        Ok(())
    }

    /// Read the LAS/LAZ file and insert all the points in the DT.
    ///
    /// :param path: full path (a string) on disk of the file to read
    /// :param optional classification: a list of class(es) to keep. If not used then all points are inserted.
    /// :param optional thinning:  the thinning factor, eg 10 will randomly pick 1/10 points from the file.
    /// :return: throws an exception if the path is invalid
    ///
    /// >>> dt = startinpy.DT()
    /// >>> dt.read_las("/home/elvis/myfile.laz")
    /// >>> OR
    /// >>> dt.read_las("/home/elvis/myfile.laz", classification=[2,6])
    /// >>> OR
    /// >>> dt.read_las("/home/elvis/myfile.laz", thinning=10, classification=[2,6])
    #[pyo3(text_signature = "($self, path, *, classification=None, thinning=1)")]
    #[args(path, py_kwargs = "**")]
    fn read_las(&mut self, path: String, py_kwargs: Option<&PyDict>) -> PyResult<()> {
        let mut c: Vec<u8> = Vec::new();
        let mut t: u32 = 1;
        if py_kwargs.is_some() {
            let tmp = py_kwargs.unwrap();
            let a = tmp.keys();
            for each in a {
                let b: String = each.extract()?;
                if (b != "classification") && (b != "thinning") {
                    let s = format!("'{}' is an invalid keyword argument for read_las()", b);
                    return Err(PyErr::new::<exceptions::PyTypeError, _>(s));
                }
            }
            if tmp.get_item("classification").is_some() {
                c = tmp.get_item("classification").unwrap().extract()?;
            }
            if tmp.get_item("thinning").is_some() {
                t = tmp.get_item("thinning").unwrap().extract()?;
            }
        }
        let re = las::Reader::from_path(path);
        if re.is_err() {
            return Err(PyErr::new::<exceptions::PyIOError, _>(
                "Invalid path for LAS/LAZ file.",
            ));
        }
        //-- make a list of classifications
        let mut classes: Vec<las::point::Classification> = Vec::new();
        for each in &c {
            let nc = Classification::new(*each);
            if nc.is_ok() {
                classes.push(nc.unwrap());
            }
        }
        let mut reader = re.unwrap();
        let _count = reader.header().number_of_points();
        let mut rng = thread_rng();
        for each in reader.points() {
            if rng.gen_ratio(1, t) == true {
                let p = each.unwrap();
                if classes.is_empty() == false {
                    if classes.contains(&p.classification) {
                        let _re = self.t.insert_one_pt(p.x, p.y, p.z);
                    }
                } else {
                    let _re = self.t.insert_one_pt(p.x, p.y, p.z);
                }
            }
        }
        Ok(())
    }

    /// Get/set the snap tolerance used to merge vertices during insertion.
    /// Two vertices closer than this will be the merged during insertion.
    /// (default=0.001)
    ///
    /// >>> dt = startinpy.DT()
    /// >>> dt.snap_tolerance = 0.05 #-- modify to 0.05unit
    /// >>> print("The snap tolerance is:", dt.snap_tolerance)
    /// The snap tolerance is: 0.05
    #[getter(snap_tolerance)]
    fn get_snap_tolerance(&self) -> PyResult<f64> {
        Ok(self.t.get_snap_tolerance())
    }

    #[setter(snap_tolerance)]
    fn set_snap_tolerance(&mut self, snaptol: f64) {
        self.t.set_snap_tolerance(snaptol);
    }

    /// :return: number of (finite) vertices    
    fn number_of_vertices(&self) -> PyResult<usize> {
        Ok(self.t.number_of_vertices())
    }

    /// :return: number of (finite) triangles    
    fn number_of_triangles(&self) -> PyResult<usize> {
        Ok(self.t.number_of_triangles())
    }

    /// Return the point for the vertex with index *vi*.
    /// An exception is thrown if vertex index is invalid.
    ///
    /// :param vi: the index of the vertex
    /// :return: the point
    ///
    /// >>> v = dt.get_point(4)
    /// [13.0, 2.0, 11.11]
    #[pyo3(text_signature = "($self, vi)")]
    fn get_point<'py>(
        &self,
        py: Python<'py>,
        vi: usize,
    ) -> PyResult<&'py PyArray<f64, numpy::Ix1>> {
        let re = self.t.get_point(vi);
        if re.is_ok() {
            return Ok(PyArray::from_vec(py, re.unwrap()));
        } else {
            return Err(PyErr::new::<exceptions::PyIndexError, _>(
                "Invalid vertex index.",
            ));
        }
    }

    /// Return the convex hull as an array of vertex indices.
    ///
    /// :return: an array of vertex indices, oriented counter-clockwise (CCW)
    #[pyo3(text_signature = "($self)")]
    fn convex_hull<'py>(&self, py: Python<'py>) -> PyResult<&'py PyArray<usize, numpy::Ix1>> {
        Ok(PyArray::from_vec(py, self.t.convex_hull()))
    }

    /// Return the bbox of the dataset
    ///
    /// :return: an array of 4 coordinates: [minx, miny, maxx, maxy]
    ///
    /// >>> bbox = dt.get_bbox()
    /// [ 505043.690 5258283.953  523361.172 5275100.003 ]
    #[pyo3(text_signature = "($self)")]
    fn get_bbox<'py>(&self, py: Python<'py>) -> PyResult<&'py PyArray<f64, numpy::Ix1>> {
        Ok(PyArray::from_vec(py, self.t.get_bbox()))
    }

    /// Is the point [x, y] located inside the convex hull of the DT.
    ///
    /// :param x: the x-coordinate
    /// :param y: the y-coordinate
    /// :return: True if [x,y] is inside the convex hull or on its boundary, False otherwise.
    #[pyo3(text_signature = "($self, x, y)")]
    fn is_inside_convex_hull(&self, x: f64, y: f64) -> PyResult<bool> {
        let re = self.t.locate(x, y);
        if re.is_ok() == true {
            return Ok(true);
        } else {
            Ok(false)
        }
    }

    /// Return True if vertex *vi* is on the boundary of the convex hull, False otherwise.
    ///
    /// :param vi: the vertex index
    /// :return: True if *vi* is on the boundary of the convex hull, False otherwise.
    ///          Also False is returned if the vertex doesn't exist in the DT.
    #[pyo3(text_signature = "($self, vi)")]
    fn is_vertex_convex_hull(&self, vi: usize) -> PyResult<bool> {
        Ok(self.t.is_vertex_convex_hull(vi))
    }

    /// Return the closest vertex index to [x, y] (distance in 2D).
    /// An Exception is thrown if [x, y] is outside the convex hull.
    ///
    /// :param x: the x-coordinate
    /// :param y: the y-coordinate
    /// :return: the vertex index of the closest point
    ///
    /// >>> try:
    /// >>>     cp = dt.closest_point(32.1, 66.9)
    /// >>> except Exception as e:
    /// >>>     print(e)
    #[pyo3(text_signature = "($self, x, y)")]
    fn closest_point(&self, x: f64, y: f64) -> PyResult<usize> {
        let re = self.t.closest_point(x, y);
        if re.is_err() {
            return Err(PyErr::new::<exceptions::PyException, _>("Outside CH"));
        } else {
            Ok(re.unwrap())
        }
    }

    /// Return the triangles incident to vertex *vi*.
    /// Exception thrown if vertex index doesn't exist in the DT.
    ///
    /// :param vi: the vertex index
    /// :return: an array of triangles (ordered counter-clockwise)
    ///
    /// >>> tri = dt.incident_triangles_to_vertex(3)
    /// >>> for i, dt in enumerate(tri):
    /// >>>     print(i, t)    
    /// 0 [3, 4, 6]
    /// 1 [3, 6, 7]
    /// 2 [3, 7, 8]
    /// 3 [3, 8, 2]
    /// 4 [3, 2, 9]
    /// 5 [3, 9, 4]
    #[pyo3(text_signature = "($self, vi)")]
    fn incident_triangles_to_vertex<'py>(
        &self,
        py: Python<'py>,
        vi: usize,
    ) -> PyResult<&'py PyArray<usize, numpy::Ix2>> {
        let re = self.t.incident_triangles_to_vertex(vi);
        if re.is_ok() {
            let l = re.unwrap();
            let mut trs: Vec<Vec<usize>> = Vec::with_capacity(l.len());
            for each in l {
                let mut tr = Vec::with_capacity(3);
                tr.push(each.v[0]);
                tr.push(each.v[1]);
                tr.push(each.v[2]);
                trs.push(tr);
            }
            return Ok(PyArray::from_vec2(py, &trs).unwrap());
        } else {
            return Err(PyErr::new::<exceptions::PyIndexError, _>(
                "Invalid vertex index.",
            ));
        }
    }

    /// Return an array of vertex indices that are adjacent to vertex *vi*,
    /// that is those on the edges incident to *vi*.
    /// An exception is thrown if *vi* does not exist in the DT.
    ///
    /// :param vi: the vertex index
    /// :return: an array of vertex indices (ordered counter-clockwise)
    #[pyo3(text_signature = "($self, vi)")]
    fn adjacent_vertices_to_vertex<'py>(
        &self,
        py: Python<'py>,
        vi: usize,
    ) -> PyResult<&'py PyArray<usize, numpy::Ix1>> {
        let re = self.t.adjacent_vertices_to_vertex(vi);
        if re.is_ok() {
            return Ok(PyArray::from_vec(py, re.unwrap()));
        } else {
            return Err(PyErr::new::<exceptions::PyIndexError, _>(
                "Invalid vertex index.",
            ));
        }
    }

    /// Verify if a triangle exists in the DT.
    ///
    /// :param t: the triangle, an array of 3 vertex indices
    /// :return: True if t exists, False otherwise.
    ///
    /// >>> re = dt.is_triangle(np.array([11, 162, 666])))
    #[pyo3(text_signature = "($self, t)")]
    fn is_triangle(&self, t: Vec<usize>) -> PyResult<bool> {
        let tr = startin::Triangle {
            v: [t[0], t[1], t[2]],
        };
        Ok(self.t.is_triangle(&tr))
    }

    /// Locate the triangle containing the point [x, y] (projected to 2D).
    /// An error is thrown if it is outside the convex hull.
    ///
    /// :param x: the x-coordinate
    /// :param y: the y-coordinate
    /// :return: the triangle.
    #[pyo3(text_signature = "($self, x, y)")]
    fn locate<'py>(
        &self,
        py: Python<'py>,
        x: f64,
        y: f64,
    ) -> PyResult<&'py PyArray<usize, numpy::Ix1>> {
        let re = self.t.locate(x, y);
        let mut tr: Vec<usize> = Vec::new();
        if re.is_ok() {
            let t = re.unwrap();
            tr.push(t.v[0]);
            tr.push(t.v[1]);
            tr.push(t.v[2]);
            return Ok(PyArray::from_vec(py, tr));
        } else {
            return Err(PyErr::new::<exceptions::PyException, _>("Outside CH"));
        }
    }

    /// Interpolation method: nearest neighbour (or closest neighbour).
    /// An Exception is thrown if [x, y] is outside the convex hull.    
    ///
    /// :param x: the x-coordinate
    /// :param y: the y-coordinate
    /// :return: the estimated value
    #[pyo3(text_signature = "($self, x, y)")]
    fn interpolate_nn(&self, x: f64, y: f64) -> PyResult<f64> {
        let re = self.t.interpolate_nn(x, y);
        if re.is_err() {
            return Err(PyErr::new::<exceptions::PyException, _>("Outside CH"));
        }
        Ok(re.unwrap())
    }

    /// Interpolation method: linear interpolation in TIN.
    /// An Exception is thrown if [x, y] is outside the convex hull.    
    ///
    /// :param x: the x-coordinate
    /// :param y: the y-coordinate
    /// :return: the estimated value
    #[pyo3(text_signature = "($self, x, y)")]
    fn interpolate_tin_linear(&self, x: f64, y: f64) -> PyResult<f64> {
        let re = self.t.interpolate_tin_linear(x, y);
        if re.is_err() {
            return Err(PyErr::new::<exceptions::PyException, _>("Outside CH"));
        }
        Ok(re.unwrap())
    }

    /// Interpolation method: Laplace interpolation (`details about the method <http://dilbert.engr.ucdavis.edu/~suku/nem/index.html>`_).
    /// This is a variation of natural interpolation method with distances used instead of stolen areas.
    /// Thus faster in practice.
    /// An Exception is thrown if [x, y] is outside the convex hull.    
    ///
    /// :param x: the x-coordinate
    /// :param y: the y-coordinate
    /// :return: the estimated value
    ///
    /// >>> try:
    /// >>>     zhat = dt.interpolate_laplace(55.2, 33.1)
    /// >>>     print("result: ", zhat)
    /// >>> except Exception as e:
    /// >>>     print(e)
    /// 64.08234343
    #[pyo3(text_signature = "($self, x, y)")]
    fn interpolate_laplace(&mut self, x: f64, y: f64) -> PyResult<f64> {
        let re = self.t.interpolate_laplace(x, y);
        if re.is_err() {
            return Err(PyErr::new::<exceptions::PyException, _>("Outside CH"));
        }
        Ok(re.unwrap())
    }

    /// Interpolation method: natural neighbour method (also called Sibson's method).
    /// An Exception is thrown if [x, y] is outside the convex hull.    
    ///
    /// :param x: the x-coordinate
    /// :param y: the y-coordinate
    /// :return: the estimated value
    ///
    /// >>> try:
    /// >>>     zhat = dt.interpolate_laplace(55.2, 33.1)
    /// >>>     print("result: ", zhat)
    /// >>> except Exception as e:
    /// >>>     print(e)
    /// 64.08234343
    #[pyo3(text_signature = "($self, x, y)")]
    fn interpolate_nni(&mut self, x: f64, y: f64) -> PyResult<f64> {
        let re = self.t.interpolate_nni(x, y);
        if re.is_err() {
            return Err(PyErr::new::<exceptions::PyException, _>("Outside CH"));
        }
        Ok(re.unwrap())
    }

    /// Write an OBJ of the DT to the path (a string).
    /// Throws an exception if the path is invalid.
    ///
    /// :param path: full path (a string) on disk of the file to create (will overwrite)
    /// :return: (nothing)
    ///
    /// >>> dt.write_obj("/home/elvis/myfile.obj")
    #[pyo3(text_signature = "($self, path)")]
    fn write_obj(&self, path: String) -> PyResult<()> {
        let re = self.t.write_obj(path.to_string());
        if re.is_err() {
            return Err(PyErr::new::<exceptions::PyIOError, _>("Invalid path"));
        }
        Ok(())
    }

    /// Write an PLY of the DT to the path (a string).
    /// Throws an exception if the path is invalid.
    ///
    /// :param path: full path (a string) on disk of the file to create (will overwrite)
    /// :return: (nothing)
    ///
    /// >>> dt.write_ply("/home/elvis/myfile.ply")
    #[pyo3(text_signature = "($self, path)")]
    fn write_ply(&self, path: String) -> PyResult<()> {
        let re = self.t.write_ply(path.to_string());
        if re.is_err() {
            return Err(PyErr::new::<exceptions::PyIOError, _>("Invalid path"));
        }
        Ok(())
    }

    /// Write a GeoJSON file of the DT (vertices+triangles) to the path (a string).
    /// Throws an exception if the path is invalid.
    ///
    /// :param path: full path (a string) on disk of the file to create (will overwrite)
    /// :return: (nothing)
    ///
    /// >>> dt.write_obj("/home/elvis/myfile.geojson")
    #[pyo3(text_signature = "($self, path)")]
    fn write_geojson(&self, path: String) -> PyResult<()> {
        let re = self.t.write_geojson(path.to_string());
        if re.is_err() {
            return Err(PyErr::new::<exceptions::PyIOError, _>("Invalid path"));
        }
        Ok(())
    }

    /// Vertically exaggerate the elevation values of the vertices.
    /// Used mostly for visualisation.
    ///
    /// :param factor: a positive value (can be <1.0 to remove exaggeration)
    /// :return: (nothing)
    ///
    /// >>> dt.vertical_exaggeration(2.0)
    /// >>> dt.vertical_exaggeration(0.5)
    #[pyo3(text_signature = "($self, factor)")]
    fn vertical_exaggeration(&mut self, factor: f64) {
        self.t.vertical_exaggeration(factor);
    }
}
