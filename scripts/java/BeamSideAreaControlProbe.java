// SPDX-License-Identifier: AGPL-3.0-or-later
// Independent development oracle: invokes the unmodified official sideAreaOf.
// No Rust code, native outputs, or native expectations are read by this probe.
import java.awt.Rectangle;
import java.awt.geom.*;
import java.lang.reflect.*;
import java.nio.file.*;
import java.util.*;

public class BeamSideAreaControlProbe {
    static Class<?> cls(String name) throws Exception {
        return Class.forName("org.audiveris.omr." + name);
    }
    static String array(double[] values) { return Arrays.toString(values); }
    static void neighbors(List<double[]> points, double x, double y) {
        for (double px : new double[]{Math.nextDown(x), x, Math.nextUp(x)})
            for (double py : new double[]{Math.nextDown(y), y, Math.nextUp(y)})
                points.add(new double[]{px, py});
    }
    @SuppressWarnings({"unchecked", "rawtypes"})
    public static void main(String[] args) throws Exception {
        Class<?> builderType = cls("sheet.beam.BeamsBuilder");
        Class<?> beamType = cls("sig.inter.AbstractBeamInter");
        Class<?> sideType = cls("util.HorizontalSide");
        Class<?> unsafeType = Class.forName("sun.misc.Unsafe");
        Field field = unsafeType.getDeclaredField("theUnsafe"); field.setAccessible(true);
        Object builder = unsafeType.getMethod("allocateInstance", Class.class)
            .invoke(field.get(null), builderType);
        Method method = builderType.getDeclaredMethod("sideAreaOf", String.class,
            beamType, sideType, double.class, double.class, double.class);
        method.setAccessible(true);
        Object impacts = cls("sig.inter.AbstractBeamInter$Impacts")
            .getConstructor(double.class, double.class, double.class,
                double.class, double.class, double.class)
            .newInstance(1., 1., 1., 1., 1., 1.);
        Constructor<?> beamConstructor = cls("sig.inter.BeamInter")
            .getConstructor(cls("sig.GradeImpacts"), Line2D.class, double.class);
        double[][] medians = {
            {10,20,30,20}, {10,20,30,30}, {10,20,30,10},
            {10.25,20.25,30.25,20.25}, {103.3,57.8,141.7,72.1},
            {-13.25,-7.5,9.75,-3.125}
        };
        // height, vertical margin, reach, overlap; includes empty/reversed areas.
        double[][] parameters = {
            {4,1,5,0}, {2,0,6,0}, {3,.25,4.5,2.5}, {0,0,5,0},
            {2,0,1,0}, {2,0,0,0}, {0,1,8,1.25}, {.1,.2,.5,.75},
            {2,0,Math.nextDown(1.),0}, {2,0,Math.nextUp(1.),0}
        };
        StringBuilder json = new StringBuilder("{\"original_method\":\"BeamsBuilder.sideAreaOf\",\"native_outputs_used\":false,\"cases\":[");
        int count = 0, queries = 0, empty = 0;
        for (double[] median : medians) for (double[] p : parameters)
            for (String side : new String[]{"LEFT", "RIGHT"}) {
                Object beam = beamConstructor.newInstance(impacts,
                    new Line2D.Double(median[0],median[1],median[2],median[3]),p[0]);
                Area area = (Area)method.invoke(builder, null, beam,
                    Enum.valueOf((Class)sideType,side),p[1],p[2],p[3]);
                Rectangle bounds = area.getBounds();
                if (area.isEmpty()) empty++;
                List<double[]> points = new ArrayList<>();
                for (int x=bounds.x-1; x<=bounds.x+bounds.width+1; x++)
                    for (int y=bounds.y-1; y<=bounds.y+bounds.height+1; y++)
                        points.add(new double[]{x,y});
                PathIterator it = area.getPathIterator(null);
                double[] segment = new double[6], first = null, previous = null;
                while (!it.isDone()) {
                    int kind = it.currentSegment(segment);
                    if (kind == PathIterator.SEG_MOVETO || kind == PathIterator.SEG_LINETO) {
                        double[] current = {segment[0], segment[1]};
                        neighbors(points,current[0],current[1]);
                        if (previous != null)
                            neighbors(points,(previous[0]+current[0])/2.,(previous[1]+current[1])/2.);
                        if (kind == PathIterator.SEG_MOVETO) first = current;
                        previous = current;
                    } else if (kind == PathIterator.SEG_CLOSE) {
                        neighbors(points,(previous[0]+first[0])/2.,(previous[1]+first[1])/2.);
                        previous = null;
                    } else throw new IllegalStateException("unexpected nonlinear segment");
                    it.next();
                }
                if (count++ != 0) json.append(',');
                json.append("{\"median\":").append(array(median))
                    .append(",\"parameters\":").append(array(p))
                    .append(",\"side\":\"").append(side)
                    .append("\",\"bounds\":").append(Arrays.toString(new int[]{bounds.x,bounds.y,bounds.width,bounds.height}))
                    .append(",\"empty\":").append(area.isEmpty()).append(",\"queries\":[");
                boolean comma = false;
                for (double[] point : points) {
                    if (comma) json.append(','); comma = true;
                    json.append('[').append(point[0]).append(',').append(point[1])
                        .append(',').append(area.contains(point[0],point[1])).append(']');
                    queries++;
                }
                json.append("]}");
            }
        json.append("],\"case_count\":").append(count)
            .append(",\"query_count\":").append(queries)
            .append(",\"empty_case_count\":").append(empty).append("}\n");
        Files.writeString(Path.of(args[0]), json);
        System.out.println("original cases="+count+" queries="+queries+" empty="+empty);
    }
}
