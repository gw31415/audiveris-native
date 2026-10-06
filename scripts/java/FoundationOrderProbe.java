// SPDX-License-Identifier: AGPL-3.0-or-later
// Unmodified original six-check/purge cascade. Declared SIG, not raster parity.
import java.awt.Rectangle;
import java.awt.geom.*;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class FoundationOrderProbe extends BeamOriginDependencyProbe {
    static final String[] CHECKS = {"checkStemEndingHeads", "checkHeads", "checkHooksHaveStem",
            "checkBeamsHaveBothStems", "checkLedgers", "checkStems"};
    static Object make(String name, Object... args) throws Exception {
        for (var ctor : c(name).getConstructors()) {
            try { return ctor.newInstance(args); } catch (IllegalArgumentException ignored) { }
        }
        throw new IllegalStateException(name);
    }
    static List<Object> state(Object head, Object stem, Object hook) throws Exception {
        List<Object> out = new ArrayList<>();
        for (Object n : List.of(head, stem, hook)) out.add(obj("kind", n.getClass().getSimpleName(),
                "removed", call(n, "isRemoved"), "contextual_grade", call(n, "getContextualGrade")));
        return out;
    }
    public static void main(String[] args) throws Exception {
        var mapper = new ObjectMapper();
        var creation = mapper.readTree(Path.of(args[1]).toFile()).get("cases").get(72);
        var b = creation.get("registered").get("bounds");
        int x = b.get(0).asInt(), y = b.get(1).asInt(), w = b.get(2).asInt(), h = b.get(3).asInt();
        double stemGrade = creation.get("first").get("grade").asDouble();
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> observations = new ArrayList<>();
        for (boolean adapter : new boolean[]{false, true}) {
            Object book = c("sheet.Book").getMethod("loadBook", Path.class).invoke(null, Path.of(args[0]));
            Object sheet = call(((List<?>)call(book, "getStubs")).get(0), "getSheet");
            Object sys = ((List<?>)call(sheet, "getSystems")).get(0), sig = call(sys, "getSig");
            for (Object n : new ArrayList<>((Collection<?>)call(sig, "vertexSet")))
                if (!(Boolean)call(n, "isRemoved")) call(n, "remove");
            Object image = Class.forName("ij.process.ByteProcessor")
                    .getConstructor(int.class,int.class,byte[].class).newInstance(w,h,new byte[w*h]);
            Object factory = make("run.RunTableFactory",c("run.Orientation").getField("VERTICAL").get(null));
            Object glyph = make("glyph.Glyph",x,y,call(factory,"createTable",image));
            Object stem = make("sig.inter.StemInter",glyph,stemGrade);
            Object head = make("sig.inter.HeadInter",new Rectangle(x-11,y+h-5,12,8),
                    c("glyph.Shape").getField("NOTEHEAD_BLACK").get(null),.4,call(sys,"getFirstStaff"),0.);
            Object hook = make("sig.inter.BeamHookInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),
                    new Line2D.Double(x-10,y,x+10,y),6.);
            call(hook,"setGrade",.8);
            for (Object n : List.of(head,stem,hook)) call(sig,"addVertex",n);
            Object hs = make("sig.relation.HeadStemRelation");
            call(hs,"setHeadSide",c("util.HorizontalSide").getField("LEFT").get(null));
            call(hs,"setExtensionPoint",new Point2D.Double(x+.5,y+h-1));
            call(hs,"setGrade",1.); call(sig,"addEdge",head,stem,hs);
            Object bs = make("sig.relation.BeamStemRelation");
            call(bs,"setBeamPortion",c("sig.relation.BeamPortion").getField("LEFT").get(null));
            call(bs,"setExtensionPoint",new Point2D.Double(x+.5,y));
            call(bs,"setGrade",1.); call(sig,"addEdge",hook,stem,bs);
            Object reducer = make("sig.SigReducer",sys,true);
            var ctor = c("sig.SigReducer$AdapterForFoundations").getDeclaredConstructor(c("sig.SigReducer"));
            ctor.setAccessible(true); Object originalAdapter = ctor.newInstance(reducer);
            call(sig,"contextualize");
            Object before = state(head,stem,hook);
            List<Object> passes = new ArrayList<>();
            for (int iteration=1;iteration<=16;iteration++) {
                int modified = 0; List<Object> checks = new ArrayList<>();
                if (adapter) modified = ((Number)call(originalAdapter,"checkConsistencies")).intValue();
                else for (String check : CHECKS) {
                    int count = ((Number)call(reducer,check)).intValue(); modified += count;
                    List<Object> purged = new ArrayList<>();
                    for (Object n : (Collection<?>)call(reducer,"contextualizeAndPurge"))
                        purged.add(n.getClass().getSimpleName());
                    checks.add(obj("check",check,"modified",count,"purged_kinds",purged,
                            "state",state(head,stem,hook)));
                }
                passes.add(obj("iteration",iteration,"modified",modified,"checks",checks));
                if (modified==0) break;
                if (iteration==16) throw new IllegalStateException("original cascade did not stabilize");
            }
            observations.add(obj("actual_original_adapter",adapter,"initial",before,"passes",passes,
                    "final",state(head,stem,hook),"physical_stem_glyph_reference_retained",call(stem,"getGlyph")==glyph,
                    "head",obj("kind","HeadInter","shape","NOTEHEAD_BLACK","bounds",rect(call(head,"getBounds")),"grade",.4,"pitch",0.,"staff_id",1),
                    "hook",obj("kind","BeamHookInter","shape","BEAM_HOOK","bounds",rect(call(hook,"getBounds")),"median",line(call(hook,"getMedian")),"grade",.8),
                    "head_relation",obj("kind","HeadStemRelation","head_side","LEFT","extension_point",point(call(hs,"getExtensionPoint")),"grade",call(hs,"getGrade"),"source_ratio",call(hs,"getSourceRatio"),"target_ratio",call(hs,"getTargetRatio"),"invading",call(hs,"isInvading")),
                    "beam_relation",obj("kind","BeamStemRelation","beam_portion","LEFT","extension_point",point(call(bs,"getExtensionPoint")),"grade",call(bs,"getGrade"),"source_ratio",call(bs,"getSourceRatio"),"target_ratio",call(bs,"getTargetRatio"))));
        }
        mapper.writerWithDefaultPrettyPrinter().writeValue(Path.of(args[2]).toFile(),
                obj("scope","declared original three-inter six-check cascade; manual trace independently compared with actual AdapterForFoundations, not whole reduceFoundations or raster parity",
                        "original_creation_case",72,"observations",observations));
        System.exit(0);
    }
}
