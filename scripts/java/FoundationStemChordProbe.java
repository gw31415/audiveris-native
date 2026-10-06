// SPDX-License-Identifier: AGPL-3.0-or-later
// Unmodified HeadChord members/leading note/locations/bounds on actual typed SIG.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class FoundationStemChordProbe extends FoundationOverlapProbe {
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        String[] names={"up","down","equal-distance-bottom-up-tie","stem-only"};
        int[][] ys={{120,140},{80,100},{86,126},{}};
        for(int ci=0;ci<names.length;ci++) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig");
            for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
            Object chord=make("sig.inter.HeadChordInter",.8);call(chord,"setStaff",call(sys,"getFirstStaff"));call(sig,"addVertex",chord);
            Object stem=make("sig.inter.StemInter",physical(100,80,2,60,"VERTICAL",null),.8);call(sig,"addVertex",stem);call(chord,"setStem",stem);
            Map<Object,String> labels=new IdentityHashMap<>();labels.put(chord,"chord");labels.put(stem,"stem");
            List<Object> nodes=new ArrayList<>();List<Object> heads=new ArrayList<>();
            for(int i=0;i<ys[ci].length;i++) {
                Object h=head(sys,physical(88,ys[ci][i],12,8,"VERTICAL",null));call(sig,"addVertex",h);call(chord,"addMember",h);heads.add(h);labels.put(h,"head"+i);
                nodes.add(obj("label",labels.get(h),"kind","HeadInter","shape","NOTEHEAD_BLACK","bounds",rect(call(h,"getBounds")),"grade",.8,"pitch",0.,"staff_id",1,"glyph",glyph(call(h,"getGlyph"))));
            }
            nodes.add(obj("label","stem","kind","StemInter","shape","STEM","bounds",rect(call(stem,"getBounds")),"grade",.8,"median",line(call(stem,"getMedian")),"width",call(stem,"getWidth"),"glyph",glyph(call(stem,"getGlyph"))));
            List<String> ordered=new ArrayList<>();for(Object h:(Collection<?>)call(chord,"getMembers"))ordered.add(labels.get(h));
            List<Object> rels=new ArrayList<>();for(Object rel:(Collection<?>)call(sig,"edgeSet"))rels.add(obj("kind",rel.getClass().getSimpleName(),"source",labels.get(call(sig,"getEdgeSource",rel)),"target",labels.get(call(sig,"getEdgeTarget",rel))));
            nodes.add(obj("label","chord","kind","HeadChordInter","bounds",rect(call(chord,"getBounds")),"grade",.8,"member_labels",ordered,"leading_label",labels.get(call(chord,"getLeadingNote")),"head_location",heads.isEmpty()?null:point(call(chord,"getHeadLocation")),"tail_location",heads.isEmpty()?null:point(call(chord,"getTailLocation")),"location_boundary",heads.isEmpty()?"original getHeadLocation/getTailLocation throws NullPointerException without leading note; not called again":"original getters returned"));
            cases.add(obj("name",names[ci],"nodes",nodes,"initial_relations",rels));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","four declared original HeadChord getMembers/getLeadingNote/computeLocations/getBounds conditions; actual typed Containment and ChordStem, not raster chord recognition or full ownership callbacks","cases",cases));System.exit(0);
    }
}
