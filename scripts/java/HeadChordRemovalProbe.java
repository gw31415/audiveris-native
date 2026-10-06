// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.awt.geom.Point2D;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class HeadChordRemovalProbe extends FoundationOverlapProbe {
    static Object observeRemoval(Object sig,Object staff,Object measure,List<Object> nodes,Map<Object,String> labels)throws Exception {
        List<Object> live=new ArrayList<>(),removed=new ArrayList<>(),rels=new ArrayList<>(),notes=new ArrayList<>(),chords=new ArrayList<>();
        for(Object n:nodes){if((Boolean)call(n,"isRemoved")){removed.add(labels.get(n));continue;}
            Map<String,Object> row=obj("label",labels.get(n),"kind",n.getClass().getSimpleName(),"bounds",rect(call(n,"getBounds")),"grade",call(n,"getGrade"));
            if(n.getClass().getSimpleName().equals("HeadChordInter")){List<Object> ms=new ArrayList<>();for(Object m:(Collection<?>)call(n,"getMembers"))ms.add(labels.get(m));row.put("members",ms);row.put("stem",labels.get(call(n,"getStem")));}
            else row.put("abnormal",call(n,"isAbnormal"));live.add(row);
        }
        for(Object e:(Collection<?>)call(sig,"edgeSet"))rels.add(obj("kind",e.getClass().getSimpleName(),"source",labels.get(call(sig,"getEdgeSource",e)),"target",labels.get(call(sig,"getEdgeTarget",e))));
        java.lang.reflect.Field f=c("sheet.Staff").getDeclaredField("notes");f.setAccessible(true);Object raw=f.get(staff);if(raw!=null)for(Object n:(Collection<?>)raw)if(labels.containsKey(n))notes.add(labels.get(n));
        for(Object n:(Collection<?>)call(measure,"getHeadChords"))if(labels.containsKey(n))chords.add(labels.get(n));
        return obj("live",live,"removed",removed,"relations",rels,"staff_notes",notes,"measure_head_chords",chords);
    }
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> cases=new ArrayList<>();
        for(boolean shared:List.of(false,true)){
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),staff=call(sys,"getFirstStaff");
            for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
            Object h=head(sys,physical(300,250,12,8,"VERTICAL",null));call(h,"setGrade",.6);call(h,"setContextualGrade",.75);
            Object s=make("sig.inter.StemInter",physical(312,200,2,58,"VERTICAL",null),.8),ch=make("sig.inter.HeadChordInter",.8);call(ch,"setStaff",staff);
            List<Object> nodes=new ArrayList<>(List.of(h,s,ch));Map<Object,String> labels=new IdentityHashMap<>();labels.put(h,"head");labels.put(s,"stem");labels.put(ch,"chord");for(Object n:nodes)call(sig,"addVertex",n);
            Object rel=make("sig.relation.HeadStemRelation");call(rel,"setHeadSide",c("util.HorizontalSide").getField("RIGHT").get(null));call(rel,"setExtensionPoint",new Point2D.Double(313,254));call(sig,"addEdge",h,s,rel);
            call(ch,"addMember",h);call(ch,"setStem",s);
            Object measure=call(call(staff,"getPart"),"getMeasureAt",call(ch,"getCenter"));call(measure,"addInter",ch);
            if(shared){Object other=make("sig.inter.HeadChordInter",.8);call(other,"setStaff",staff);nodes.add(other);labels.put(other,"other");call(sig,"addVertex",other);call(other,"addMember",h);call(other,"setStem",s);call(measure,"addInter",other);}
            Object before=observeRemoval(sig,staff,measure,nodes,labels);call(ch,"remove");Object after=observeRemoval(sig,staff,measure,nodes,labels);call(ch,"remove");Object repeated=observeRemoval(sig,staff,measure,nodes,labels);
            cases.add(obj("name",shared?"shared-member":"exclusive-member","before",before,"after",after,"repeat_is_idempotent",new ObjectMapper().valueToTree(after).equals(new ObjectMapper().valueToTree(repeated))));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","two declared original extensive HeadChord removal cases; actual Containment/ChordStem/HeadStem, Staff notes and Measure registration; no Voice/lyrics/cache hydration or raster parity","cases",cases));System.exit(0);
    }
}
