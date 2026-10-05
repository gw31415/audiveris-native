// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only local original NaturalSpline/GeoPath/ReversePathIterator/Area control.
// Boundary assembly follows pinned StaffManager.getGlobalLine and SystemManager.computeSystemArea.
import java.awt.Shape;
import java.awt.geom.*;
import java.lang.reflect.*;
import java.util.*;
public class BeamAreaBoundaryProbe {
 static Class<?> cls(String n)throws Exception{return Class.forName("org.audiveris.omr.math."+n);}
 static Object spline(double[][] points)throws Exception{double[] x=new double[points.length],y=x.clone();for(int i=0;i<x.length;i++){x[i]=points[i][0];y[i]=points[i][1];}return cls("NaturalSpline").getMethod("interpolate",double[].class,double[].class).invoke(null,x,y);}
 static PathIterator global(double[][] points,double shift)throws Exception{Object s=spline(points);Path2D path=(Path2D)cls("GeoPath").getConstructor().newInstance();path.moveTo(0,points[0][1]);path.append((Shape)s,true);path.lineTo(100,points[points.length-1][1]);return path.getPathIterator(AffineTransform.getTranslateInstance(0,shift));}
 static Area area(double[][] points,boolean north)throws Exception{PathIterator top=north?global(points,4):new Line2D.Double(0,0,100,0).getPathIterator(null);PathIterator bottom=north?new Line2D.Double(0,140,100,140).getPathIterator(null):global(points,36);Path2D path=(Path2D)cls("GeoPath").getConstructor().newInstance();path.append(top,false);PathIterator reverse=(PathIterator)cls("ReversePathIterator").getConstructor(PathIterator.class).newInstance(bottom);path.append(reverse,true);return new Area(path);}
 static String points(double[][] points){List<String> out=new ArrayList<>();for(double[] p:points)out.add("["+p[0]+","+p[1]+"]");return "["+String.join(",",out)+"]";}
 public static void main(String[] args)throws Exception{double[][][] sets={{{10,30},{90,50}},{{10,50},{90,30}},{{10,40},{90,40}},{{10,30.5},{90,50.5}},{{10,30},{50,50},{90,30}},{{10,50},{50,30},{90,50}},{{10,30},{30,40},{50,30},{70,40},{90,30}},{{10,30},{11,40},{80,30},{90,40}}};String[] names={"ascending-line","descending-line","horizontal-line","half-pixel-line","quadratic-maximum","quadratic-minimum","cubic-oscillation","nonmonotone-x-cubic"};List<String> cases=new ArrayList<>();for(int n=0;n<sets.length;n++){double[][] p=sets[n];Area north=area(p,true),south=area(p,false);List<String> query=new ArrayList<>();for(int x=10;x<=90;x++)for(int y=20;y<=95;y++){boolean a=south.contains(x,y),b=north.contains(x,y);query.add("["+x+","+y+","+a+","+b+"]");}cases.add("{\"name\":\""+names[n]+"\",\"points\":"+points(p)+",\"queries\":["+String.join(",",query)+"]}");}System.out.println("{\"upstream_commit\":\"9e1e55cd2746037d059345881c53e6a6754bffbd\",\"native_outputs_used\":false,\"query_fields\":[\"x\",\"y\",\"system1\",\"system2\"],\"cases\":["+String.join(",",cases)+"]}");}
}
