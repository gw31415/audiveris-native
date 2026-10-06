// SPDX-License-Identifier: AGPL-3.0-or-later
// Declared original cleaner lifecycle, not automatic multi-stem recognition.
import java.awt.Rectangle;
import java.awt.geom.Point2D;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class HeadCleanerLifecycleProbe extends FoundationOverlapProbe {
    static Object state(Object head,List<Object> stems,List<Object> rels) throws Exception {
        List<Object> kept=new ArrayList<>(),abnormal=new ArrayList<>(),removed=new ArrayList<>();
        Object sig=call(head,"getSig");
        for(int i=0;i<rels.size();i++)if((Boolean)call(sig,"containsEdge",rels.get(i)))kept.add(i);
        for(Object s:stems){abnormal.add(call(s,"isAbnormal"));removed.add(call(s,"isRemoved"));}
        return obj("kept_relation_indexes",kept,"head_abnormal",call(head,"isAbnormal"),
            "stem_abnormal",abnormal,"stem_removed",removed,"head_removed",call(head,"isRemoved"));
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        for(String name:List.of("manual-frozen-three","parallel-first-high-ratio","parallel-first-low-ratio","manual-small-canonical"))
        for(String mode:List.of("selected","automatic")) {
            // Automatic parallel/high-ratio already threw in original5.11
            // (deleted-edge alias in isCanonicalShare). Preserve its separate
            // limit observation; do not repeat it or invent a successful result.
            if(name.startsWith("parallel")&&mode.equals("automatic"))continue;
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
            for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
            boolean marked=name.startsWith("manual"),canonical=name.endsWith("canonical"),parallel=name.startsWith("parallel");
            String shape=canonical?"NOTEHEAD_BLACK_SMALL":"NOTEHEAD_BLACK";
            Object head=make("sig.inter.HeadInter",new Rectangle(40,50,15,12),c("glyph.Shape").getField(shape).get(null),.8,call(sys,"getFirstStaff"),0.);
            call(head,"setManual",marked);if(marked)call(head,"freeze");call(sig,"addVertex",head);
            List<Object> stems=new ArrayList<>(),rels=new ArrayList<>(),inputs=new ArrayList<>(),connections=new ArrayList<>();
            int count=name.equals("manual-frozen-three")?3:2;
            for(int i=0;i<count;i++) {
                int y=canonical&&i==1?21:50;
                Object g=physical(40+5*i,y,2,40,"VERTICAL",null);
                double grade=canonical?.8:parallel?(name.endsWith("low-ratio")?(i==0?.8:.4):(i==0?.2:.8)):(i==0?.2:i==1?.8:.4);
                Object stem=make("sig.inter.StemInter",g,grade);
                call(stem,"setManual",marked);if(marked)call(stem,"freeze");call(sig,"addVertex",stem);stems.add(stem);
                inputs.add(obj("kind","StemInter","shape","STEM","glyph",glyph(g),"bounds",rect(call(stem,"getBounds")),
                    "median",line(call(stem,"getMedian")),"width",call(stem,"getWidth"),"grade",grade,"manual",marked,"frozen",marked));
                int multiplicity=parallel&&i==0?2:1;
                for(int j=0;j<multiplicity;j++) {
                    Object r=make("sig.relation.HeadStemRelation");
                    String side=canonical&&i==1?"RIGHT":"LEFT";
                    call(r,"setHeadSide",c("util.HorizontalSide").getField(side).get(null));
                    call(r,"setExtensionPoint",new Point2D.Double(41+5*i,50));
                    call(r,"setInOutGaps",0.,0.,0);
                    if(parallel&&i==0&&((name.endsWith("low-ratio")&&j==0)||(name.endsWith("high-ratio")&&j==1)))call(r,"setGrade",.25);
                    call(r,"setManual",marked);call(sig,"addEdge",head,stem,r);rels.add(r);
                    connections.add(obj("stem_index",i,"kind","HeadStemRelation","head_side",side,"extension_point",point(call(r,"getExtensionPoint")),
                        "dx",call(r,"getDx"),"dy",call(r,"getDy"),"grade",call(r,"getGrade"),"source_ratio",call(r,"getSourceRatio"),
                        "target_ratio",call(r,"getTargetRatio"),"manual",marked));
                }
            }
            Object before=state(head,stems,rels);
            List<Integer> selected=new ArrayList<>();
            for(Object stem:stems)selected.add(rels.indexOf(call(sig,"getRelation",head,stem,c("sig.relation.HeadStemRelation"))));
            Object retriever=make("sheet.stem.StemsRetriever",sys),checker=make("sheet.stem.HeadStemsCleaner",head);
            List<Object> partitions=new ArrayList<>();List<Object> all=new ArrayList<>();
            for(Object r:rels)all.add(call(sig,"getEdgeTarget",r));
            for(Object p:(List<?>)call(sig,"getPartitions",null,all)) {
                List<Integer> indexes=new ArrayList<>();for(Object s:(List<?>)p)indexes.add(stems.indexOf(s));partitions.add(indexes);
            }
            if(mode.equals("selected"))call(checker,"check",stems);else call(retriever,"checkHeadStems",head);
            Object after=state(head,stems,rels);
            if(mode.equals("selected"))call(checker,"check",stems);else call(retriever,"checkHeadStems",head);
            cases.add(obj("name",name,"mode",mode,"head",obj("kind","HeadInter","shape",shape,"bounds",rect(call(head,"getBounds")),
                "grade",.8,"pitch",0.,"staff_id",1,"manual",marked,"frozen",marked),"stems",inputs,"connections",connections,
                "original_first_relation_indexes",selected,"automatic_partition_stem_indexes",partitions,"before",before,"after_first",after,
                "after_second",state(head,stems,rels),"all_physical_glyphs_retained",stems.stream().allMatch(s->{try{return call(s,"getGlyph")!=null;}catch(Exception e){throw new RuntimeException(e);}})));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope",
            "six declared original selected/manual/checkHeadStems lifecycle conditions; automatic parallel cases excluded following separately observed original exception; not raster recognition or complete cleaner/epilog parity","cases",cases));
        System.exit(0);
    }
}
