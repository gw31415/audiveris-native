// SPDX-License-Identifier: AGPL-3.0-or-later
// Declared original analyzeChords controls, separate from image export parity.
import java.awt.Rectangle;
import java.awt.geom.Point2D;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class FoundationChordProbe extends FoundationOrderProbe {
    public static void main(String[] args) throws Exception {
        var mapper = new ObjectMapper();
        var creation = mapper.readTree(Path.of(args[1]).toFile()).get("cases").get(72);
        var b = creation.get("registered").get("bounds");
        int x=b.get(0).asInt(),y=b.get(1).asInt(),w=b.get(2).asInt(),h=b.get(3).asInt();
        double grade=creation.get("first").get("grade").asDouble();
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        for (boolean same : new boolean[]{true,false}) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
            for (Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))
                if (!(Boolean)call(n,"isRemoved")) call(n,"remove");
            Object image=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class)
                    .newInstance(w,h,new byte[w*h]);
            Object table=call(make("run.RunTableFactory",c("run.Orientation").getField("VERTICAL").get(null)),"createTable",image);
            Object stem=make("sig.inter.StemInter",make("glyph.Glyph",x,y,table),grade);
            List<Object> heads=new ArrayList<>(),nodes=new ArrayList<>(),connections=new ArrayList<>();
            Map<Object,String> labels=new IdentityHashMap<>(); labels.put(stem,"stem");
            for (int i=0;i<3;i++) {
                String shape=i==1&&!same?"NOTEHEAD_VOID":"NOTEHEAD_BLACK";
                Rectangle bounds=new Rectangle(i==2?x:x-11,i==0?y+h-5:y+12,12,8);
                double hg=i==0?.4:(i==1?.8:.6);
                Object head=make("sig.inter.HeadInter",bounds,c("glyph.Shape").getField(shape).get(null),hg,call(sys,"getFirstStaff"),0.);
                heads.add(head);labels.put(head,"head"+i);call(sig,"addVertex",head);
                nodes.add(obj("label","head"+i,"kind","HeadInter","shape",shape,"bounds",rect(bounds),"grade",hg,"pitch",0.,"staff_id",1));
            }
            call(sig,"addVertex",stem);
            nodes.add(obj("label","stem","kind","StemInter","shape","STEM","bounds",rect(call(stem,"getBounds")),"median",line(call(stem,"getMedian")),"grade",grade));
            for (int i=0;i<2;i++) {
                Object rel=make("sig.relation.HeadStemRelation");
                call(rel,"setHeadSide",c("util.HorizontalSide").getField("RIGHT").get(null));
                call(rel,"setExtensionPoint",new Point2D.Double(x+.5,i==0?y+h-1:y+16));
                call(rel,"setGrade",1.);call(sig,"addEdge",heads.get(i),stem,rel);
                connections.add(obj("kind","HeadStemRelation","source","head"+i,"target","stem","head_side","RIGHT",
                        "extension_point",point(call(rel,"getExtensionPoint")),"grade",call(rel,"getGrade"),"source_ratio",call(rel,"getSourceRatio"),"target_ratio",call(rel,"getTargetRatio"),"invading",call(rel,"isInvading")));
            }
            Object reducer=make("sig.SigReducer",sys,true);call(sig,"contextualize");call(reducer,"analyzeChords");
            List<Object> relations=new ArrayList<>();
            for (Object rel:(Collection<?>)call(sig,"edgeSet")) {
                String kind=rel.getClass().getSimpleName();
                if (!kind.equals("Exclusion")&&!kind.equals("HeadHeadRelation")) continue;
                Map<String,Object> data=obj("kind",kind,"source",labels.get(call(sig,"getEdgeSource",rel)),"target",labels.get(call(sig,"getEdgeTarget",rel)));
                if (kind.equals("Exclusion")) data.put("cause",get(rel,"cause").toString());
                else { data.put("grade",call(rel,"getGrade"));data.put("source_ratio",call(rel,"getSourceRatio"));data.put("target_ratio",call(rel,"getTargetRatio")); }
                relations.add(data);
            }
            call(sig,"contextualize");List<Object> contextual=new ArrayList<>();
            for (Object n:heads) contextual.add(obj("label",labels.get(n),"contextual_grade",call(n,"getContextualGrade")));
            contextual.add(obj("label","stem","contextual_grade",call(stem,"getContextualGrade")));
            int edgeCount=((Collection<?>)call(sig,"edgeSet")).size();call(reducer,"analyzeChords");
            cases.add(obj("name",same?"same-duration":"different-duration","nodes",nodes,"head_connections",connections,
                    "new_relations",relations,"contextual",contextual,"repeat_is_idempotent",edgeCount==((Collection<?>)call(sig,"edgeSet")).size()));
        }
        mapper.writerWithDefaultPrettyPrinter().writeValue(Path.of(args[2]).toFile(),obj("scope","two declared original ordinary analyzeChords/HeadHead/INCOMPATIBLE/intersected-unlinked-head conditions; not raster or whole reduction parity","cases",cases));
        System.exit(0);
    }
}
