// SPDX-License-Identifier: AGPL-3.0-or-later
// Declared original live-SIG checkStems conditions, not automatic image outcomes.
import java.awt.Rectangle;
import java.awt.geom.*;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class StemConsistencyProbe extends BeamOriginDependencyProbe {
    static Object make(String name, Object... args) throws Exception {
        for (var ctor : c(name).getConstructors()) {
            try { return ctor.newInstance(args); }
            catch (IllegalArgumentException ignored) { }
        }
        throw new IllegalStateException(name);
    }
    static Object solidGlyph() throws Exception {
        Object image = Class.forName("ij.process.ByteProcessor")
                .getConstructor(int.class, int.class, byte[].class)
                .newInstance(2, 61, new byte[122]);
        Object table = call(make("run.RunTableFactory", c("run.Orientation").getField("VERTICAL").get(null)), "createTable", image);
        return make("glyph.Glyph", 200, 150, table);
    }
    static Object connection(Object r) throws Exception {
        Map<String,Object> out = obj("kind", r.getClass().getSimpleName(), "grade", call(r,"getGrade"),
                "extension_point", point(call(r,"getExtensionPoint")),
                "source_ratio", call(r,"getSourceRatio"), "target_ratio", call(r,"getTargetRatio"));
        if (r.getClass().getSimpleName().equals("HeadStemRelation")) {
            out.put("head_side", call(r,"getHeadSide").toString());
            out.put("invading", call(r,"isInvading"));
        } else out.put("beam_portion", call(r,"getBeamPortion").toString());
        return out;
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases = new ArrayList<>();
        for (String name : List.of("no-heads", "top-strongest", "bottom-strongest", "beam-strongest", "unknown-direction")) {
            Object book = c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet = call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object sys = ((List<?>)call(sheet,"getSystems")).get(0), sig = call(sys,"getSig");
            for (Object v : new ArrayList<>((Collection<?>)call(sig,"vertexSet")))
                if (v.getClass().getSimpleName().equals("StemInter")) call(v,"remove");
            Object stem = c("sig.inter.StemInter").getConstructor(c("glyph.Glyph"),Double.class)
                    .newInstance(solidGlyph(),.8);
            call(sig,"addVertex",stem); call(stem,"setContextualGrade",.8);
            List<Object> heads = new ArrayList<>(), links = new ArrayList<>(), nodes = new ArrayList<>(), relations = new ArrayList<>();
            if (!name.equals("no-heads")) {
                boolean middle = name.equals("unknown-direction");
                for (int i=0; i<(middle?1:2); i++) {
                    int y = middle?176:(i==0?146:206);
                    double grade = middle?.8:(i==0?(name.equals("top-strongest")?.9:.7):.8);
                    Object h = c("sig.inter.HeadInter").getConstructor(Rectangle.class,c("glyph.Shape"),Double.class,c("sheet.Staff"),Double.class)
                            .newInstance(new Rectangle(i==0&&!middle?188:200,y,12,8), c("glyph.Shape").getField("NOTEHEAD_BLACK").get(null),grade,call(sys,"getFirstStaff"),0.);
                    call(sig,"addVertex",h);call(h,"setContextualGrade",grade);
                    Object r = make("sig.relation.HeadStemRelation");
                    call(r,"setHeadSide",c("util.HorizontalSide").getField(i==0&&!middle?"LEFT":"RIGHT").get(null));
                    call(r,"setExtensionPoint",new Point2D.Double(200.5,y+4));
                    call(r,"setGrade",0.);call(sig,"addEdge",h,stem,r);
                    nodes.add(obj("kind","HeadInter","shape","NOTEHEAD_BLACK","bounds",rect(call(h,"getBounds")),"grade",grade,"contextual_grade",grade,"pitch",0.,"staff_id",1));
                    relations.add(connection(r)); heads.add(h); links.add(r);
                }
            }
            Object beam = null, beamRelation = null;
            if (name.equals("beam-strongest")) {
                beam = make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(190,210,230,210),6.);
                call(sig,"addVertex",beam);call(beam,"setGrade",.99);call(beam,"setContextualGrade",.99);
                beamRelation = make("sig.relation.BeamStemRelation");
                call(beamRelation,"setBeamPortion",c("sig.relation.BeamPortion").getField("LEFT").get(null));
                call(beamRelation,"setExtensionPoint",new Point2D.Double(200.5,210));
                call(beamRelation,"setGrade",0.);call(sig,"addEdge",beam,stem,beamRelation);
            }
            int dir = ((Number)call(stem,"computeDirection")).intValue();
            Object geometry = obj("kind","StemInter","shape","STEM","glyph",glyph(call(stem,"getGlyph")),"bounds",rect(call(stem,"getBounds")),"median",line(call(stem,"getMedian")),"grade",.8,"contextual_grade",.8);
            Object reducer = make("sig.SigReducer",sys,true);
            Object modified = call(reducer,"checkStems");
            List<Object> retained = new ArrayList<>(), excluded = new ArrayList<>();
            for (int i=0;i<heads.size();i++) {
                retained.add(call(sig,"containsEdge",links.get(i)));
                boolean conflict = false;
                for (Object e : (Collection<?>)call(sig,"edgeSet")) if(e.getClass().getSimpleName().equals("Exclusion")) {
                    Object a=call(sig,"getEdgeSource",e),b=call(sig,"getEdgeTarget",e);
                    if ((a==heads.get(i)&&b==stem)||(b==heads.get(i)&&a==stem)) conflict=true;
                }
                excluded.add(conflict);
            }
            cases.add(obj("name",name,"stem",geometry,"heads",nodes,"head_relations",relations,
                    "beam",beam==null?null:obj("kind","BeamInter","shape","BEAM","bounds",rect(call(beam,"getBounds")),"median",line(call(beam,"getMedian")),"grade",.99,"contextual_grade",.99),
                    "beam_relation",beamRelation==null?null:connection(beamRelation),"direction",dir,"modified",modified,"stem_removed",call(stem,"isRemoved"),"retained_head_relations",retained,"head_stem_exclusions",excluded));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("native_outputs_used",false,"scope","five declared original ordinary live-SIG checkStems/computeDirection conditions on saved page; no automatic raster outcome claim","cases",cases));
        System.exit(0);
    }
}
