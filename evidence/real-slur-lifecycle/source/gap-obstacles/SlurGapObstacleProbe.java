// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.Point;
import java.awt.Rectangle;
import java.awt.geom.*;
import java.nio.file.*;
import java.util.*;

public class SlurGapObstacleProbe extends NativeSlurCandidateMathProbe {
    public static void main(String[] args) throws Exception {
        Object builder = allocate(c("sheet.curve.SlursBuilder"));
        List<Object> rows = new ArrayList<>();
        for (String line : Files.readAllLines(Path.of(args[0]))) {
            if (line.isBlank()) continue;
            String[] cols = line.split("\\t");
            String[] xy = cols[1].split(","), box = cols[2].split(",");
            Point a = new Point(Integer.parseInt(xy[0]), Integer.parseInt(xy[1]));
            Point b = new Point(Integer.parseInt(xy[2]), Integer.parseInt(xy[3]));
            Rectangle bounds = new Rectangle(Integer.parseInt(box[0]), Integer.parseInt(box[1]), Integer.parseInt(box[2]), Integer.parseInt(box[3]));
            Rectangle extension = new Rectangle(a.x, a.y, 1, 1);
            extension.add(b);
            Area strip = (Area)call(builder, "getLineArea", a, b);
            Area obstacle = null;
            if (!cols[3].equals("null")) {
                String[] m = cols[3].split(",");
                obstacle = (Area)c("math.AreaUtil").getMethod("horizontalParallelogram", Point2D.class, Point2D.class, double.class).invoke(null,
                    new Point2D.Double(Double.parseDouble(m[0]), Double.parseDouble(m[1])),
                    new Point2D.Double(Double.parseDouble(m[2]), Double.parseDouble(m[3])), Double.parseDouble(m[4]));
            }
            boolean extBounds = bounds.intersects(extension);
            boolean extArea = obstacle == null || obstacle.intersects(extension);
            boolean boundsLine = bounds.intersectsLine(a.x, a.y, b.x, b.y);
            boolean areaCross = obstacle == null || (Boolean)c("math.AreaUtil").getMethod("intersection", Area.class, Area.class).invoke(null, strip, obstacle);
            Object sheet = allocate(c("sheet.Sheet"));
            Object manager = c("sheet.SystemManager").getConstructor(c("sheet.Sheet")).newInstance(sheet);
            Object system = c("sheet.SystemInfo").getConstructor(int.class,c("sheet.Sheet"),List.class).newInstance(1,sheet,null);
            call(system,"setArea",new Area(new Rectangle(-100,-100,10000,10000)));
            set(manager,"systems",new ArrayList<>(List.of(system)));
            set(sheet,"systemManager",manager);
            Object inter = allocate(c(obstacle==null ? "sig.inter.HeadInter" : "sig.inter.BeamInter"));
            set(inter,"bounds",bounds);
            set(inter,"area",obstacle);
            Object skeleton = allocate(c("sheet.curve.Skeleton"));
            Map<Object,List<Object>> erased = new HashMap<>();
            erased.put(system,List.of(inter));
            set(skeleton,"nonCrossables",erased);
            set(builder,"sheet",sheet);
            set(builder,"skeleton",skeleton);
            set(builder,"reverse",false);
            Object curve = c("sheet.curve.SlurInfo").getConstructor(int.class,Point.class,Point.class,List.class,c("sheet.curve.Model"),Collection.class,int.class).newInstance(1,null,null,List.of(a),null,List.of(),66);
            Object arc = c("sheet.curve.Arc").getConstructor(Point.class,Point.class,List.class,c("sheet.curve.Model")).newInstance(null,null,List.of(b),null);
            Object view = call(curve,"getArcView",arc,false);
            Set<Object> reachable = new LinkedHashSet<>(List.of(view));
            call(builder,"filterReachableArcs",curve,reachable);
            rows.add(map("case", cols[0], "extension_box", List.of(extension.x,extension.y,extension.width,extension.height),
                "bounds_intersect_extension",extBounds,"area_intersects_extension",extArea,"bounds_intersect_segment",boundsLine,
                "full_endpoint_strip_intersects_area",areaCross,"reject_pair",extBounds&&extArea&&boundsLine&&areaCross,
                "original_filter_rejects",reachable.isEmpty(),"null_area",obstacle==null,"strip_bounds",List.of(strip.getBounds().x,strip.getBounds().y,strip.getBounds().width,strip.getBounds().height)));
        }
        Object head = allocate(c("sig.inter.HeadInter"));
        List<Object> grades = new ArrayList<>();
        for(double grade:new double[]{0.399999999,0.4,0.599999999,0.6}) {set(head,"grade",grade);grades.add(map("grade",grade,"is_contextually_good",call(head,"isContextuallyGood")));}
        Files.writeString(Path.of(args[1]),json(map("scope","Original CurvesBuilder.getLineArea, Rectangle extension/line operations and AreaUtil.horizontalParallelogram/intersection plus actual private filterReachableArcs on detached real original Sheet/System/Inter/Curve/Arc owners with one reachable Arc and explicitly supplied erased-owner list. Constructors bypassed where needed; no Skeleton erasure lifecycle, recognition, Book or native owner proof.","head_good_grade_controls",grades,"rows",rows))+"\n");
    }
}
