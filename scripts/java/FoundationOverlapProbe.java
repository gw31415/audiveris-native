// SPDX-License-Identifier: AGPL-3.0-or-later
// Original detectOverlaps on declared ordinary SIG objects, not image parity.
import java.awt.Rectangle;
import java.awt.geom.Line2D;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class FoundationOverlapProbe extends FoundationOrderProbe {
    static Object physical(int x, int y, int w, int h, String orientation, int[][] points) throws Exception {
        byte[] pixels = new byte[w*h]; Arrays.fill(pixels,(byte)255);
        if (points==null) Arrays.fill(pixels,(byte)0);
        else for (int[] p:points) pixels[p[1]*w+p[0]]=0;
        Object image=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(w,h,pixels);
        Object factory=make("run.RunTableFactory",c("run.Orientation").getField(orientation).get(null));
        return make("glyph.Glyph",x,y,call(factory,"createTable",image));
    }
    static Object head(Object sys,Object glyph) throws Exception {
        Object h=make("sig.inter.HeadInter",(Rectangle)call(glyph,"getBounds"),c("glyph.Shape").getField("NOTEHEAD_BLACK").get(null),.8,call(sys,"getFirstStaff"),0.);
        call(h,"setGlyph",glyph);return h;
    }
    static Object beam(double[] m,double height,Object glyph,boolean hook) throws Exception {
        Object b=make(hook?"sig.inter.BeamHookInter":"sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(m[0],m[1],m[2],m[3]),height);
        call(b,"setGrade",.8);if(glyph!=null)call(b,"setGlyph",glyph);return b;
    }
    static Object bar(double[] m,double width,Object glyph) throws Exception {
        return make("sig.inter.BarlineInter",glyph,c("glyph.Shape").getField("THIN_BARLINE").get(null),.8,new Line2D.Double(m[0],m[1],m[2],m[3]),width);
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        String[] names={"disjoint-pixels","shared-pixel","existing-support","standard-head-stem","compatible-beams","head-core-border","implicit-head","stem-cross-orientation","barline-physical-glyph","area-positive","area-boundary-only"};
        List<Object> cases=new ArrayList<>();
        for (String name:names) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
            for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
            Object hg=physical(100,100,12,8,"VERTICAL",new int[][]{{0,0},{11,0},{0,7},{11,7}});
            int[][] bp={{6,0},{6,7},{0,4},{11,4}};
            if(!name.equals("disjoint-pixels"))bp=new int[][]{{6,0},{6,7},{0,4},{11,4},{0,0}};
            Object bg=physical(100,100,12,8,"HORIZONTAL",bp);
            Object a,b;
            if(name.startsWith("area-")) {
                a=beam(new double[]{100,100,112,112},2.,null,false);
                b=bar(name.equals("area-positive")?new double[]{103,102,107,106}:new double[]{103,101,107,105},2.,null);
            } else if(name.equals("stem-cross-orientation")) {
                a=make("sig.inter.StemInter",physical(100,100,3,12,"VERTICAL",null),.8);
                b=make("sig.inter.StemInter",physical(101,100,3,12,"HORIZONTAL",null),.8);
            } else if(name.equals("barline-physical-glyph")) {
                a=bar(new double[]{106,99,106,109},2.,physical(105,99,2,10,"VERTICAL",null));
                b=beam(new double[]{100,104,112,104},8.,bg,false);
            } else {
                a=head(sys,hg);
                if(name.equals("standard-head-stem"))b=make("sig.inter.StemInter",physical(100,100,6,8,"VERTICAL",null),.8);
                else if(name.equals("head-core-border"))b=beam(new double[]{100,104,101,104},8.,physical(100,100,1,8,"VERTICAL",null),true);
                else b=beam(new double[]{100,104,112,104},8.,bg,false);
                if(name.equals("compatible-beams"))a=beam(new double[]{100,104,112,104},8.,bg,true);
                if(name.equals("implicit-head"))call(a,"setImplicit",true);
            }
            call(sig,"addVertex",a);call(sig,"addVertex",b);
            List<Object> support=new ArrayList<>();
            if(name.equals("existing-support")){
                Object r=make("sig.relation.BeamHeadRelation",1.);
                call(sig,"addEdge",b,a,r);
                support.add(obj("kind","BeamHeadRelation","source","b","target","a",
                    "grade",call(r,"getGrade"),"source_ratio",call(r,"getSourceRatio"),"target_ratio",call(r,"getTargetRatio")));
            }
            call(sig,"contextualize");
            List<Object> beforeContext=List.of(call(a,"getContextualGrade"),call(b,"getContextualGrade"));
            List<Object> nodes=new ArrayList<>();Map<Object,String> labels=new IdentityHashMap<>();labels.put(a,"a");labels.put(b,"b");
            for(Object n:List.of(a,b)) {
                String kind=n.getClass().getSimpleName();
                Map<String,Object> row=obj("label",labels.get(n),"kind",kind,"shape",call(n,"getShape").toString(),"bounds",rect(call(n,"getBounds")),"core_bounds",rect(call(n,"getCoreBounds")),"grade",call(n,"getGrade"),"glyph",glyph(call(n,"getGlyph")),"implicit",call(n,"isImplicit"));
                if(kind.equals("HeadInter")){row.put("pitch",0.);row.put("staff_id",1);}
                else {row.put("median",line(call(n,"getMedian")));row.put(kind.contains("Beam")?"height":"width",call(n,kind.contains("Beam")?"getHeight":"getWidth"));}
                nodes.add(row);
            }
            Object reducer=make("sig.SigReducer",sys,true);
            List<Object> filtered=new ArrayList<>();for(Object n:List.of(a,b))if(!(Boolean)call(n,"isImplicit"))filtered.add(n);
            filtered.sort(Comparator.comparingInt(n->((Rectangle)callUnchecked(n,"getBounds")).x));
            call(reducer,"detectOverlaps",filtered);
            List<Object> exclusions=new ArrayList<>();
            for(Object e:(Collection<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("Exclusion"))exclusions.add(obj("source",labels.get(call(sig,"getEdgeSource",e)),"target",labels.get(call(sig,"getEdgeTarget",e)),"cause",get(e,"cause").toString()));
            int count=((Collection<?>)call(sig,"edgeSet")).size();call(reducer,"detectOverlaps",filtered);
            call(sig,"contextualize");
            cases.add(obj("name",name,"nodes",nodes,"support",support,"directional_overlaps",List.of(call(a,"overlaps",b),call(b,"overlaps",a)),"exclusions",exclusions,"repeat_is_idempotent",count==((Collection<?>)call(sig,"edgeSet")).size(),
                "contextual_grades_before",beforeContext,"contextual_grades_after",List.of(call(a,"getContextualGrade"),call(b,"getContextualGrade"))));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","declared ordinary original detectOverlaps; glyph/core/support/implicit/standard-head-stem/beam compatibility/line-area branches, not raster or full reduction parity","cases",cases));
        System.exit(0);
    }
    static Object callUnchecked(Object o,String method){try{return call(o,method);}catch(Exception e){throw new RuntimeException(e);}}
}
