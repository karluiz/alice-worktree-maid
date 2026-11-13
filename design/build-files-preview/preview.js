const tabs = [...document.querySelectorAll('[data-tab]')]
const panels = [...document.querySelectorAll('[data-panel]')]
const dialog = document.querySelector('[data-dialog]')
const dialogBox = dialog.querySelector('.cleanup-dialog')
const safetyContent = document.querySelector('.safety-content')
const result = document.querySelector('.cleanup-result')
let returnFocus = null

function selectTab(name) {
  tabs.forEach(tab => tab.setAttribute('aria-selected', String(tab.dataset.tab === name)))
  panels.forEach(panel => { panel.hidden = panel.dataset.panel !== name })
}

tabs.forEach(tab => tab.addEventListener('click', () => selectTab(tab.dataset.tab)))
document.querySelectorAll('[data-show-worktrees]').forEach(link => {
  link.addEventListener('click', () => selectTab('worktrees'))
})

const expand = document.querySelector('[data-target-expand]')
const evidence = document.querySelector('.target-evidence')
expand.addEventListener('click', () => {
  const open = expand.getAttribute('aria-expanded') !== 'true'
  expand.setAttribute('aria-expanded', String(open))
  evidence.hidden = !open
})

const check = document.querySelector('.target-check')
check.addEventListener('click', () => {
  const selected = check.getAttribute('aria-pressed') !== 'true'
  check.setAttribute('aria-pressed', String(selected))
  check.setAttribute('aria-label', `${selected ? 'Deselect' : 'Select'} qhipa-ai target`)
})

function openCleanup() {
  returnFocus = document.activeElement
  result.hidden = true
  safetyContent.hidden = false
  dialog.hidden = false
  window.setTimeout(() => dialogBox.focus(), 250)
}

document.querySelectorAll('[data-open-cleanup]').forEach(link => {
  link.addEventListener('click', openCleanup)
})

function closeCleanup() {
  dialog.hidden = true
  if (returnFocus instanceof HTMLElement) returnFocus.focus()
}

document.querySelector('[data-close-cleanup]').addEventListener('click', closeCleanup)
document.querySelector('[data-confirm-cleanup]').addEventListener('click', () => {
  dialog.hidden = true
  safetyContent.hidden = true
  result.hidden = false
  result.querySelector('button').focus()
})
document.querySelector('[data-reset-preview]').addEventListener('click', () => {
  result.hidden = true
  safetyContent.hidden = false
  dialog.hidden = false
  dialogBox.focus()
})

document.addEventListener('keydown', event => {
  if (dialog.hidden) return
  if (event.key === 'Escape') {
    event.preventDefault()
    closeCleanup()
    return
  }
  if (event.key !== 'Tab') return
  const controls = [...dialogBox.querySelectorAll('button, a[href], [tabindex="0"]')]
  const first = controls[0]
  const last = controls[controls.length - 1]
  if (event.shiftKey && (document.activeElement === first || document.activeElement === dialogBox)) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first.focus()
  }
})

const previewState = new URLSearchParams(window.location.search).get('state')
if (previewState === 'confirm') {
  document.querySelector('#safety').scrollIntoView()
  openCleanup()
} else if (previewState === 'result') {
  document.querySelector('#safety').scrollIntoView()
  dialog.hidden = true
  safetyContent.hidden = true
  result.hidden = false
}
