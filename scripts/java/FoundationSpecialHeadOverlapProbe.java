// SPDX-License-Identifier: AGPL-3.0-or-later
// Declared original special-head overlaps, not new whole-page admission.
import java.awt.Rectangle;
import java.awt.geom.Point2D;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class FoundationSpecialHeadOverlapProbe extends FoundationOverlapProbe {
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        for(String shape:List.of("NOTEHEAD_BLACK_SMALL","NOTEHEAD_VOID_SMALL","WHOLE_NOTE_SMALL","BREVE_SMALL","NOTEHEAD_CROSS","BREVE")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
            for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
            Object hg=physical(100,100,12,8,"VERTICAL",null);
            Object a=make("sig.inter.HeadInter",new Rectangle(100,100,12,8),c("glyph.Shape").getField(shape).get(null),.8,call(sys,"getFirstStaff"),0.);
            call(a,"setGlyph",hg);
            Object b=make("sig.inter.StemInter",physical(100,100,6,8,"VERTICAL",null),.8);
            call(sig,"addVertex",a);call(sig,"addVertex",b);
            List<Object> support=new ArrayList<>();
            if(shape.equals("NOTEHEAD_VOID_SMALL")) {
                Object rel=make("sig.relation.HeadStemRelation");
                call(rel,"setHeadSide",c("util.HorizontalSide").getField("RIGHT").get(null));
                call(rel,"setExtensionPoint",new Point2D.Double(103,104));call(rel,"setGrade",1.);
                call(sig,"addEdge",a,b,rel);
                support.add(obj("kind","HeadStemRelation","source","a","target","b","grade",call(rel,"getGrade"),"source_ratio",call(rel,"getSourceRatio"),"target_ratio",call(rel,"getTargetRatio")));
            }
            call(sig,"contextualize");List<Object> before=List.of(call(a,"getContextualGrade"),call(b,"getContextualGrade"));
            List<Object> nodes=new ArrayList<>();
            for(Object n:List.of(a,b)) {
                boolean isHead=n==a;
                Map<String,Object> row=obj("label",isHead?"a":"b","kind",n.getClass().getSimpleName(),"shape",call(n,"getShape").toString(),"bounds",rect(call(n,"getBounds")),"core_bounds",rect(call(n,"getCoreBounds")),"grade",.8,"glyph",glyph(call(n,"getGlyph")),"implicit",false);
                if(isHead){row.put("pitch",0.);row.put("staff_id",1);}else{row.put("median",line(call(n,"getMedian")));row.put("width",call(n,"getWidth"));}
                nodes.add(row);
            }
            Object reducer=make("sig.SigReducer",sys,true);List<Object> sorted=List.of(a,b);call(reducer,"detectOverlaps",sorted);
            List<Object> exclusions=new ArrayList<>();
            for(Object rel:(Collection<?>)call(sig,"edgeSet"))if(rel.getClass().getSimpleName().equals("Exclusion"))exclusions.add(obj("source",call(sig,"getEdgeSource",rel)==a?"a":"b","target",call(sig,"getEdgeTarget",rel)==b?"b":"a","cause",get(rel,"cause").toString()));
            int count=((Collection<?>)call(sig,"edgeSet")).size();call(reducer,"detectOverlaps",sorted);call(sig,"contextualize");
            cases.add(obj("name",shape,"shape_is_small_head",call(c("glyph.Shape").getField(shape).get(null),"isSmallHead"),"nodes",nodes,"support",support,"directional_overlaps",List.of(call(a,"overlaps",b),call(b,"overlaps",a)),"exclusions",exclusions,"repeat_is_idempotent",count==((Collection<?>)call(sig,"edgeSet")).size(),"contextual_grades_before",before,"contextual_grades_after",List.of(call(a,"getContextualGrade"),call(b,"getContextualGrade"))));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","six declared original special-head/stem pixel/core/size/support conditions; not raster or whole-page special-head MusicXML parity","cases",cases));
        System.exit(0);
    }
}
