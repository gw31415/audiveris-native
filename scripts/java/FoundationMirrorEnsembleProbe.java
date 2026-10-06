// SPDX-License-Identifier: AGPL-3.0-or-later
// Original owned head mirrors and chord-member overlap predicates, not full LINKS.
import java.awt.Rectangle;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

public class FoundationMirrorEnsembleProbe extends FoundationOverlapProbe {
    static Object note(Object sys,int x) throws Exception {return head(sys,physical(x,100,12,8,"VERTICAL",null));}
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        for(String name:List.of("head-no-mirror","head-mirror","mirror-chord-notes","ensemble-gap","ensemble-overlap","ensemble-member-support","mirrored-chords")) {
            if(args.length>2&&!name.equals(args[2]))continue;
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
            for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
            List<Object> all=new ArrayList<>();Map<Object,String> labels=new IdentityHashMap<>();
            Object a=note(sys,100),b=note(sys,name.equals("ensemble-gap")?130:102);all.add(a);all.add(b);labels.put(a,"a");labels.put(b,"b");
            for(Object n:all)call(sig,"addVertex",n);
            if(name.equals("head-mirror")||name.equals("mirror-chord-notes")||name.equals("mirrored-chords"))call(sig,"addEdge",b,a,make("sig.relation.MirrorRelation"));
            if(name.equals("mirrored-chords")) {
                for(Object member:List.of(a,b)) {
                    Object chord=make("sig.inter.HeadChordInter",.8);call(chord,"setStaff",call(sys,"getFirstStaff"));call(sig,"addVertex",chord);call(chord,"addMember",member);all.add(chord);labels.put(chord,member==a?"chord_a":"chord_b");
                }
            }
            if(name.startsWith("ensemble")||name.equals("mirror-chord-notes")) {
                Object chord=make("sig.inter.HeadChordInter",.8);call(chord,"setStaff",call(sys,"getFirstStaff"));call(sig,"addVertex",chord);
                if(name.equals("mirror-chord-notes")) {
                    Object d=note(sys,103);call(sig,"addVertex",d);all.add(d);labels.put(d,"d");call(chord,"addMember",b);call(chord,"addMember",d);
                }else{call(chord,"addMember",a);call(chord,"addMember",b);}
                all.add(chord);labels.put(chord,"chord");
                if(name.startsWith("ensemble")) {
                    int x=name.equals("ensemble-gap")?120:100;
                    Object beam=beam(new double[]{x,104,x+12,104},8.,physical(x,100,12,8,"HORIZONTAL",null),false);
                    call(sig,"addVertex",beam);all.add(beam);labels.put(beam,"beam");
                    if(name.equals("ensemble-member-support"))for(Object h:List.of(a,b))call(sig,"addEdge",beam,h,make("sig.relation.BeamHeadRelation",1.));
                }
            }
            List<Object> nodes=new ArrayList<>(),relations=new ArrayList<>(),directions=new ArrayList<>();
            for(Object n:all) {
                String kind=n.getClass().getSimpleName();Object shape=call(n,"getShape");
                Map<String,Object> row=obj("label",labels.get(n),"kind",kind,"shape",shape==null?null:shape.toString(),"bounds",rect(call(n,"getBounds")),"core_bounds",rect(call(n,"getCoreBounds")),"grade",call(n,"getGrade"),"glyph",glyph(call(n,"getGlyph")),"implicit",false);
                if(kind.equals("HeadInter")){row.put("pitch",0.);row.put("staff_id",1);row.put("mirror_label",labels.get(call(n,"getMirror")));row.put("chord_label",labels.get(call(n,"getChord")));}
                else if(kind.equals("HeadChordInter")){List<String> members=new ArrayList<>();for(Object m:(Collection<?>)call(n,"getMembers"))members.add(labels.get(m));row.put("member_labels",members);row.put("mirror_label",labels.get(call(n,"getMirror")));}
                else{row.put("median",line(call(n,"getMedian")));row.put("height",call(n,"getHeight"));}
                nodes.add(row);
                List<Object> ds=new ArrayList<>();for(Object other:all)ds.add(n==other?null:call(n,"overlaps",other));directions.add(ds);
            }
            for(Object rel:(Collection<?>)call(sig,"edgeSet")) {
                String kind=rel.getClass().getSimpleName();Map<String,Object> row=obj("kind",kind,"source",labels.get(call(sig,"getEdgeSource",rel)),"target",labels.get(call(sig,"getEdgeTarget",rel)));
                if(kind.endsWith("HeadRelation")||kind.equals("MirrorRelation")){row.put("grade",call(rel,"getGrade"));row.put("source_ratio",call(rel,"getSourceRatio"));row.put("target_ratio",call(rel,"getTargetRatio"));}
                relations.add(row);
            }
            List<Object> sorted=new ArrayList<>(all);sorted.sort(Comparator.comparingInt(n->((Rectangle)callUnchecked(n,"getBounds")).x));
            Object reducer=make("sig.SigReducer",sys,true);call(reducer,"detectOverlaps",sorted);List<Object> exclusions=new ArrayList<>();
            for(Object rel:(Collection<?>)call(sig,"edgeSet"))if(rel.getClass().getSimpleName().equals("Exclusion"))exclusions.add(obj("source",labels.get(call(sig,"getEdgeSource",rel)),"target",labels.get(call(sig,"getEdgeTarget",rel)),"cause",get(rel,"cause").toString()));
            int count=((Collection<?>)call(sig,"edgeSet")).size();call(reducer,"detectOverlaps",sorted);
            cases.add(obj("name",name,"nodes",nodes,"initial_relations",relations,"directional_overlaps",directions,"exclusions",exclusions,"repeat_is_idempotent",count==((Collection<?>)call(sig,"edgeSet")).size()));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","declared original typed Head mirrors / stemless HeadChord containment and member pixel/support predicates; not automatic mirror recognition, stem-bound chord geometry, complete LINKS or page SIG parity","cases",cases));
        System.exit(0);
    }
}
