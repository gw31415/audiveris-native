// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent original middleArea controls; never reads Rust or native outputs.
import java.awt.Rectangle;
import java.awt.geom.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;

public class BeamMiddleAreaControlProbe extends BeamSideAreaControlProbe {
    public static void main(String[] args) throws Exception {
        Class<?> builderType = cls("sheet.beam.BeamsBuilder");
        Class<?> beamType = cls("sig.inter.AbstractBeamInter");
        Class<?> unsafeType = Class.forName("sun.misc.Unsafe");
        Field field = unsafeType.getDeclaredField("theUnsafe"); field.setAccessible(true);
        Object builder = unsafeType.getMethod("allocateInstance", Class.class)
            .invoke(field.get(null), builderType);
        Method method = builderType.getDeclaredMethod("middleArea", beamType, beamType);
        method.setAccessible(true);
        Object impacts = cls("sig.inter.AbstractBeamInter$Impacts")
            .getConstructor(double.class, double.class, double.class,
                double.class, double.class, double.class)
            .newInstance(1., 1., 1., 1., 1., 1.);
        Constructor<?> constructor = cls("sig.inter.BeamInter")
            .getConstructor(cls("sig.GradeImpacts"), Line2D.class, double.class);
        double[][][] pairs = {
            {{0,10,10,10},{14,10,34,10}},
            {{0,0,10,5},{14,9,34,19}},
            {{0,20,10,15},{14,11,34,1}},
            {{10.25,20.25,30.25,23.5},{31.75,26.125,54.25,20.75}},
            {{-23.25,-7.5,-13.25,-3.125},{-9.75,.25,9.75,8.625}},
            {{0,10,10,10},{10,10,20,10}},
            {{0,10,10,11},{10,20,20,22}},
            {{0,10,10,10},{8,11,20,15}},
            {{0,10,10,12},{0,11,20,15}},
            {{0,10,30,25},{10,12,20,15}}
        };
        double[][] heights = {{2,8},{4,4},{0,0},{0,4},{.1,.3}};
        StringBuilder json = new StringBuilder("{\"original_method\":\"BeamsBuilder.middleArea\",\"native_outputs_used\":false,\"cases\":[");
        int count=0, queries=0, empty=0;
        for (double[][] pair:pairs) for (double[] h:heights) for (int order=0;order<2;order++) {
            double[] a=pair[order], b=pair[1-order];
            double ah=h[order], bh=h[1-order];
            Object first=constructor.newInstance(impacts,new Line2D.Double(a[0],a[1],a[2],a[3]),ah);
            Object second=constructor.newInstance(impacts,new Line2D.Double(b[0],b[1],b[2],b[3]),bh);
            Area area=(Area)method.invoke(builder,first,second);
            Rectangle bounds=area.getBounds(); if(area.isEmpty())empty++;
            List<double[]> points=new ArrayList<>();
            for(int x=bounds.x-1;x<=bounds.x+bounds.width+1;x++)
                for(int y=bounds.y-1;y<=bounds.y+bounds.height+1;y++)points.add(new double[]{x,y});
            PathIterator it=area.getPathIterator(null);
            double[] segment=new double[6], start=null, previous=null;
            while(!it.isDone()) {
                int kind=it.currentSegment(segment);
                if(kind==PathIterator.SEG_MOVETO || kind==PathIterator.SEG_LINETO) {
                    double[] current={segment[0],segment[1]}; neighbors(points,current[0],current[1]);
                    if(previous!=null)neighbors(points,(previous[0]+current[0])/2.,(previous[1]+current[1])/2.);
                    if(kind==PathIterator.SEG_MOVETO)start=current;
                    previous=current;
                } else if(kind==PathIterator.SEG_CLOSE) {
                    neighbors(points,(previous[0]+start[0])/2.,(previous[1]+start[1])/2.); previous=null;
                } else throw new IllegalStateException("unexpected nonlinear segment");
                it.next();
            }
            if(count++!=0)json.append(',');
            json.append("{\"first\":").append(array(a)).append(",\"second\":").append(array(b))
                .append(",\"heights\":").append(array(new double[]{ah,bh}))
                .append(",\"bounds\":").append(Arrays.toString(new int[]{bounds.x,bounds.y,bounds.width,bounds.height}))
                .append(",\"empty\":").append(area.isEmpty()).append(",\"queries\":[");
            boolean comma=false;
            for(double[] p:points) {
                if(comma)json.append(',');comma=true;
                json.append('[').append(p[0]).append(',').append(p[1]).append(',').append(area.contains(p[0],p[1])).append(']');queries++;
            }
            json.append("]}");
        }
        json.append("],\"case_count\":").append(count).append(",\"query_count\":").append(queries)
            .append(",\"empty_case_count\":").append(empty).append("}\n");
        Files.writeString(Path.of(args[0]),json);
        System.out.println("original middle cases="+count+" queries="+queries+" empty="+empty);
    }
}
