// SPDX-License-Identifier: AGPL-3.0-or-later
// Declared original full analyzeChords size/duration families, not page parity.
import java.awt.Rectangle;
import java.awt.geom.Line2D;
import java.awt.geom.Point2D;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class FoundationSpecialChordProbe extends FoundationOverlapProbe {
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> semantics=new ArrayList<>(),cases=new ArrayList<>();
        for(String prefix:List.of("BREVE","WHOLE_NOTE","NOTEHEAD_VOID","NOTEHEAD_BLACK")) {
            String[] shapes=prefix.equals("NOTEHEAD_BLACK")?new String[]{prefix,prefix+"_SMALL","NOTEHEAD_CROSS","NOTEHEAD_DIAMOND_FILLED","NOTEHEAD_TRIANGLE_DOWN_FILLED","NOTEHEAD_CIRCLE_X"}:
                prefix.equals("NOTEHEAD_VOID")?new String[]{prefix,prefix+"_SMALL","NOTEHEAD_CROSS_VOID","NOTEHEAD_DIAMOND_VOID","NOTEHEAD_TRIANGLE_DOWN_VOID","NOTEHEAD_CIRCLE_X_VOID"}:
                new String[]{prefix,prefix+"_SMALL",prefix+"_CROSS",prefix+"_DIAMOND",prefix+"_TRIANGLE_DOWN",prefix+"_CIRCLE_X"};
            for(String s:shapes){Object shape=c("glyph.Shape").getField(s).get(null);semantics.add(obj("shape",s,"ordinal",call(shape,"ordinal"),"small",call(shape,"isSmallHead"),"duration",call(shape,"getNoteDuration").toString()));}
        }
        Object allow=c("sheet.stem.BeamLinker").getMethod("allowSmallHeadOnStandardBeam").invoke(null);
        String[] names={"cross-same-duration","standard-small-same-duration","diamond-same-void-duration","small-different-duration","small-beam-standard-head","standard-beam-small-head","mixed-beam-sizes","small-beam-small-heads"};
        String[][] shapes={{"NOTEHEAD_BLACK","NOTEHEAD_CROSS"},{"NOTEHEAD_BLACK","NOTEHEAD_BLACK_SMALL"},{"NOTEHEAD_VOID","NOTEHEAD_DIAMOND_VOID"},{"NOTEHEAD_BLACK_SMALL","NOTEHEAD_VOID_SMALL"},{"NOTEHEAD_BLACK"},{"NOTEHEAD_BLACK_SMALL"},{"NOTEHEAD_BLACK_SMALL"},{"NOTEHEAD_BLACK_SMALL","NOTEHEAD_BLACK_SMALL"}};
        String[][] beamKinds={{},{},{},{},{"SmallBeamInter"},{"BeamInter"},{"BeamInter","SmallBeamInter"},{"SmallBeamInter"}};
        for(int ci=0;ci<names.length;ci++) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
            for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
            List<Object> all=new ArrayList<>(),nodes=new ArrayList<>(),connections=new ArrayList<>();Map<Object,String> labels=new IdentityHashMap<>();
            Object stem=make("sig.inter.StemInter",physical(100,80,2,60,"VERTICAL",null),.8);
            for(int i=0;i<shapes[ci].length;i++) {
                Object head=make("sig.inter.HeadInter",new Rectangle(88,132-i*20,12,8),c("glyph.Shape").getField(shapes[ci][i]).get(null),i==0?.4:.8,call(sys,"getFirstStaff"),0.);
                call(sig,"addVertex",head);all.add(head);labels.put(head,"head"+i);
            }
            call(sig,"addVertex",stem);all.add(stem);labels.put(stem,"stem");
            for(int i=0;i<shapes[ci].length;i++) {
                Object r=make("sig.relation.HeadStemRelation");call(r,"setHeadSide",c("util.HorizontalSide").getField("RIGHT").get(null));call(r,"setExtensionPoint",new Point2D.Double(100.5,136-i*20));call(r,"setGrade",1.);call(sig,"addEdge",all.get(i),stem,r);
                connections.add(obj("kind","HeadStemRelation","source","head"+i,"target","stem","head_side","RIGHT","extension_point",point(call(r,"getExtensionPoint")),"grade",call(r,"getGrade"),"source_ratio",call(r,"getSourceRatio"),"target_ratio",call(r,"getTargetRatio"),"invading",call(r,"isInvading")));
            }
            for(int i=0;i<beamKinds[ci].length;i++) {
                boolean small=beamKinds[ci][i].equals("SmallBeamInter");double height=small?3.:6.;int y=90+i*8;
                Object beam=make("sig.inter."+beamKinds[ci][i],make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(95,y,115,y),height);call(beam,"setGrade",.8);call(beam,"setGlyph",physical(95,y-(int)height/2,20,(int)height,"HORIZONTAL",null));call(sig,"addVertex",beam);all.add(beam);labels.put(beam,"beam"+i);
                Object r=make("sig.relation.BeamStemRelation");call(r,"setBeamPortion",c("sig.relation.BeamPortion").getField("CENTER").get(null));call(r,"setExtensionPoint",new Point2D.Double(100.5,y));call(r,"setGrade",1.);call(sig,"addEdge",beam,stem,r);
                connections.add(obj("kind","BeamStemRelation","source","beam"+i,"target","stem","beam_portion","CENTER","extension_point",point(call(r,"getExtensionPoint")),"grade",call(r,"getGrade"),"source_ratio",call(r,"getSourceRatio"),"target_ratio",call(r,"getTargetRatio")));
            }
            for(Object n:all) {
                String kind=n.getClass().getSimpleName();Map<String,Object> row=obj("label",labels.get(n),"kind",kind,"shape",call(n,"getShape").toString(),"bounds",rect(call(n,"getBounds")),"grade",call(n,"getGrade"),"glyph",glyph(call(n,"getGlyph")));
                if(kind.equals("HeadInter")){row.put("pitch",0.);row.put("staff_id",1);}else{row.put("median",line(call(n,"getMedian")));row.put(kind.equals("StemInter")?"width":"height",call(n,kind.equals("StemInter")?"getWidth":"getHeight"));}nodes.add(row);
            }
            Object reducer=make("sig.SigReducer",sys,true);call(sig,"contextualize");call(reducer,"analyzeChords");List<Object> relations=new ArrayList<>(),contextual=new ArrayList<>();
            for(Object rel:(Collection<?>)call(sig,"edgeSet")) {
                String kind=rel.getClass().getSimpleName();if(!kind.equals("Exclusion")&&!kind.equals("HeadHeadRelation"))continue;
                Map<String,Object> row=obj("kind",kind,"source",labels.get(call(sig,"getEdgeSource",rel)),"target",labels.get(call(sig,"getEdgeTarget",rel)));
                if(kind.equals("Exclusion"))row.put("cause",get(rel,"cause").toString());else{row.put("grade",call(rel,"getGrade"));row.put("source_ratio",call(rel,"getSourceRatio"));row.put("target_ratio",call(rel,"getTargetRatio"));}relations.add(row);
            }
            call(sig,"contextualize");for(Object n:all)contextual.add(obj("label",labels.get(n),"contextual_grade",call(n,"getContextualGrade")));
            int count=((Collection<?>)call(sig,"edgeSet")).size();call(reducer,"analyzeChords");cases.add(obj("name",names[ci],"nodes",nodes,"head_connections",connections,"new_relations",relations,"contextual",contextual,"repeat_is_idempotent",count==((Collection<?>)call(sig,"edgeSet")).size()));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","eight declared original analyzeChords duration/size/head-beam families and24 Shape scalar semantics; original default allowSmallHeadOnStandardBeam only, no raster or full-stage parity","allowSmallHeadOnStandardBeam",allow,"shape_semantics",semantics,"cases",cases));System.exit(0);
    }
}
