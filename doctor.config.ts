import type { ReactDoctorConfig } from 'react-doctor/api'

export default {
  ignore: {
    rules: [
      'react-doctor/no-giant-component',
      'react-doctor/no-react19-deprecated-apis',
      'react-doctor/only-export-components',
    ],
  },
} satisfies ReactDoctorConfig
