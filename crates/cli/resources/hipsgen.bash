#!/bin/bash

usage(){
  # See https://tldp.org/HOWTO/Bash-Prompt-HOWTO/x405.html
  local su=`tput smul` # Start Underline
  local eu=`tput rmul` # End  Underline
  local sn=`tput sgr0` # Start normal
  local sb=`tput bold` # Start bold
  local eb=${sn}       # End   bold

cat << EOF
Script executing all the command lines required to build a HIPS catalog from
a FITS binary table. For large files, we advise to proceed step by step.

${sb}${su}Usage:${eu} $(basename ${0})${eb} [OPTIONS] <INPUT> <RA> <DEC> <OUTPUT>

${sb}${su}Arguments:${eu}${eb}
  <INPUT>   Input FITS file containing a single BINTABLE
              - FITS-plus recommanded (see TOPCAT/STILTS)
              - variable length columns (FITS HEAP) not supported
  <RA>      Index of the RA column (starting at 1)
	      - see output of 'fitstable info MYTABLE' (1st column, +1)
  <DEC>     Index of the Dec column (starting at 1)
	      - see output of 'fitstable info MYTABLE', (1st column, +1)
  <OUTPUT>  The output directory, storing generated HIPS files

${sb}${su}Options:${eu}${eb}
  ${sb}-n, --n1${eb} <N>
          Number of sources at level 1 (if allsky) [default: 3000]
  ${sb}-r, --r21${eb} <R21>
          Ratio between the number of source in level 2 and level [default: 3]
  ${sb}-m, --nt${eb} <N_T>
          From level 3, number of cells per tile [default: 500]
  ${sb}-s, --score${eb} <EXPRESSION>
          Score, if any: sources with the lower score appear first in the hierarchy
      ${sb}--<PROPERTY>${eb} <VALUE>
          See list of properties related options in 'fitstable mkhips --help'
  ${sb}-l, --log${eb} <LEVEL>
          Set the log level: off, trace, debug, info, warn, error [default: off]
      ${sb}--no-clean${eb}
          Do not remove intermediary products (sorted FITS, index, intermediary representation)
  ${sb}-h, --help${eb}
          Print help message

${sb}${su}Examples:${eu}${eb}
   ./$(basename ${0}) table.fits 1 2 table.hips
   ./$(basename ${0}) --score "Gmag" table.fits 1 2 table.hips
   ./$(basename ${0}) --log warn  --score "cavg(Jmag,Hmag,Kmag)" table.fits 1 2 table.hips
   ./$(basename ${0}) table.fits 1 2 table.hips \\
     --log trace --no-clean \\
     --score 'cavg(\${Jmag},\${Hmag},\${Kmag})' \\
     --publisher-id ivo://CDS \\
     --obs-ack "Project of institute XXX, funded by YYY"
EOF
}


# PARSE OPTIONS
POSITIONAL_ARGS=()
# * Action
HELP="false"  # -h, --help
# * Algo options
N1=""
R2=""
NT=""
SCORE=""
OTHER=()
LOG="off"
CLEAN="true"

while [[ $# -gt 0 ]]; do
  case $1 in
    -n|--n1)
      N1="--n1 $2"
      shift # past argument
      shift # past value
      ;;
    -r|--r21)
      R2="--r21 $2"
      shift # past argument
      shift # past value
      ;;
    -m|--nt)
      NT="-m $2"
      shift # past argument
      shift # past value
      ;;
    -s|--score)
      SCORE="--score $2"
      shift # past argument
      shift # past value
      ;;
    --no-clean)
      CLEAN="false"
      shift # past argument
      ;;
    -l|--log)
      LOG="$2"
      shift # past argument
      shift # past value
      ;;      
    -h|--help)
      HELP="true"
      shift # past argument
      ;;
   -*|--*)
      OTHER+=($1 "$2")
      shift # past argument
      shift # past value
      ;;
    *)
      POSITIONAL_ARGS+=("$1") # save positional arg
      shift # past argument
      ;;
  esac
done

set -- "${POSITIONAL_ARGS[@]}" # restore positional parameters

[[ ${HELP} == "true" ]] && { usage; exit 0; }

# Params
# 1: message to be print
yesno_exit() {
  while true; do
    read -p "$1 [y/n]: " yn
    case ${yn} in
      [Yy]*) return 0  ;;
      [Nn]*) echo "Aborted" ; exit  1 ;;
    esac
  done
}

# Do the job

INPUT=$1
RA=$2
DEC=$3
OUTPUT=$4

[[ "${INPUT}" == "" && "${RA}" == "" && "${DEC}" == "" && "${OUTPUT}" == "" ]] && { usage; exit 2; }

echo -n "Check for 'fitstable' in the current directory..."
CMD="./fitstable"
${CMD} 2> /dev/null
if [[ $? != 2 ]]; then
  echo " not found!"
  echo -n "Check for 'fitstable' in the path..."
  CMD="fitstable"
  ${CMD} 2> /dev/null
  if [[ $? != 2 ]]; then
    echo " not found!"
    echo "See installation: https://github.com/cds-astro/cds-fitstable-rust/blob/main/crates/cli/doc/hipsgen.md"
    exit 1
  else
    echo " found!"
  fi
else
  echo " found!"
fi

echo ""
echo "Arguments:"
echo "* Input FITS file : ${INPUT}"
echo "* RA  column index: ${RA}"
echo "* Dec column index: ${DEC}"
echo "* Output directory : ${OUTPUT}"

[[ "${INPUT}" == "" || "${RA}" == "" || "${DEC}" == "" || "${OUTPUT}" == "" ]] && { echo 'ERROR: at least one argument is missing.'; exit 2; }

echo "Check input '.fits' extension..."
[[ ${INPUT} != *.fits ]] && { echo "ERROR: input file extension is not '.fits'"; exit 1; }

echo "Create output dir if necessary..."
echo "> mkdir -p ${OUTPUT}"
mkdir -p ${OUTPUT}

DIRNAME="$(dirname ${INPUT})"
FILENAME="$(basename ${INPUT})"
# Remove the '.fits' extension
FILENAME="${FILENAME%.*}"
SORTED="${OUTPUT}/${FILENAME}.sorted.fits"
[[ -s ${SORTED} ]] && { yesno_exit "File ${SORTED} already exists, it will be overwritten. Continue?"; }

echo "Sort input FITS file..."
echo "> RUST_LOG=${LOG} ${CMD} sort ${INPUT} ${SORTED} --lon ${RA} --lat ${DEC}"
RUST_LOG=${LOG} ${CMD} sort ${INPUT} ${SORTED} --lon ${RA} --lat ${DEC}
[[ $? != 0 ]] && { echo "ERROR: exit status not 0"; exit 1; }

HCIDX="${OUTPUT}/${FILENAME}.sorted.hcidx.fits"
echo "Create sorted file index..."
echo "> RUST_LOG=${LOG} fitstable mkidx ${SORTED} ${HCIDX} --lon ${RA} --lat ${DEC}"
RUST_LOG=${LOG} fitstable mkidx ${SORTED} ${HCIDX} --lon ${RA} --lat ${DEC}
[[ $? != 0 ]] && { echo "ERROR: exit status not 0"; exit 1; }

HIPSDIR="${OUTPUT}/compact"
[[ -d "${HIPSDIR}" ]] && { yesno_exit "Dir ${HIPSDIR} already exists. Files will be overwritten and then the full directory will be removed!. Continue?"; }
echo "Build HIPS intermediary (compact) representation..."
echo "> RUST_LOG=${LOG} fitstable mkhips ${N1} ${R2} ${NT} ${SCORE} $(printf '%q ' "${OTHER[@]}") ${HCIDX} ${HIPSDIR}"
RUST_LOG=${LOG} fitstable mkhips ${N1} ${R2} ${NT} ${SCORE} "${OTHER[@]}" ${HCIDX} ${HIPSDIR}
[[ $? != 0 ]] && { echo "ERROR: exit status not 0"; exit 1; }

echo "Build standard products from intermediary representation..."
echo "* check for previous results..."
if [[ $(ls ${OUTPUT}/Norder*) != "" ]]; then
  yesno_exit "NorderXX directories found in ${OUTPUT}, remove all ${OUTPUT}/NorderXX directories (WARNING)?";
  rm -r ${OUTPUT}/Norder*
fi
echo "* build 'properties' file..."
RUST_LOG=${LOG} fitstable qhips ${HIPSDIR} properties > ${OUTPUT}/properties
[[ $? != 0 ]] && { echo "ERROR: exit status not 0"; exit 1; }

echo "* build 'Metadata.xml' file..."
RUST_LOG=${LOG} fitstable qhips ${HIPSDIR} metadata   > ${OUTPUT}/Metadata.xml
[[ $? != 0 ]] && { echo "ERROR: exit status not 0"; exit 1; }

echo "* build 'Moc.fits' file..."
RUST_LOG=${LOG} fitstable qhips ${HIPSDIR} moc        > ${OUTPUT}/Moc.fits
[[ $? != 0 ]] && { echo "ERROR: exit status not 0"; exit 1; }

echo "* build 'Norder1/Allsky.tsv' file..."
mkdir ${OUTPUT}/Norder1
RUST_LOG=${LOG} fitstable qhips ${HIPSDIR} allsky 1   > ${OUTPUT}/Norder1/Allsky.tsv
[[ $? != 0 ]] && { echo "ERROR: exit status not 0"; exit 1; }

echo "* build 'Norder2/Allsky.tsv' file..."
mkdir ${OUTPUT}/Norder2
RUST_LOG=${LOG} fitstable qhips ${HIPSDIR} allsky 2   > ${OUTPUT}/Norder2/Allsky.tsv
[[ $? != 0 ]] && { echo "ERROR: exit status not 0"; exit 1; }

echo "* build all tiles..."
[[ -f .hipscat_fifo ]] && { rm .hipscat_fifo; }
mkfifo .hipscat_fifo
RUST_LOG=${LOG} fitstable qhips ${HIPSDIR} list | tail -n +2 > .hipscat_fifo &
while read line; do
  IFS=',' read -ra array <<< "${line}"
  depth="${array[0]}"
  icell="${array[1]}"
  div10k=$((icell / 10000))
  dest="${OUTPUT}/Norder${depth}/Dir$((div1Ok * 10000))"
  [[ ! -d ${dest} ]] && { mkdir -p ${dest}; }
  RUST_LOG=${LOG} fitstable qhips ${HIPSDIR} tile ${depth} ${icell} > ${dest}/Npix${icell}.tsv
  [[ $? != 0 ]] && { echo "ERROR: exit status not 0"; exit 1; }
done < .hipscat_fifo
rm .hipscat_fifo

if [[ ${CLEAN} == "true" ]]; then
  echo "Remove temporary files..."
  echo "* Remove intermediary representation..."
  rm -r ${HIPSDIR}
  echo "* Remove sorted FITS file index..."
  rm ${HCIDX}
  echo "* Remove sorted FITS file..."
  rm ${SORTED}
fi
