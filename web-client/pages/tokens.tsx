import type { NextPage } from 'next';
import {
  Button,
  Dialog,
  DialogBody,
  DialogFooter,
  Divider,
  H3,
  HTMLTable,
  InputGroup,
} from '@blueprintjs/core';
import useTranslation from 'next-translate/useTranslation';
import { useState } from 'react';
import { createToken } from '../utils/api';
import { Token } from '../interfaces/Token';

const Tokens: NextPage = () => {
  const { t } = useTranslation('common');
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [tokenName, setTokenName] = useState('');
  const [createdTokens, setCreatedTokens] = useState<Token[]>([]);

  return (
    <div>
      <main className="flex flex-row justify-center">
        <div className="flex flex-col items-strech w-full max-w-2xl">
          <div className="flex flex-row mt-6 items-end justify-between">
            <H3 className="m-0">{t('tokens')}</H3>
            <Button
              intent="primary"
              large
              onClick={() => {
                setIsDialogOpen(true);
              }}
            >
              {t('createToken')}
            </Button>
            <Dialog
              isOpen={isDialogOpen}
              title={t('createToken')}
              icon="info-sign"
              onClose={() => {
                setIsDialogOpen(false);
              }}
            >
              <DialogBody>
                <InputGroup
                  placeholder={t('tokenName')}
                  value={tokenName}
                  onChange={(event) => {
                    setTokenName(event.target.value);
                  }}
                />
              </DialogBody>
              <DialogFooter
                actions={
                  <Button
                    disabled={tokenName === ''}
                    intent="primary"
                    text={t('create')}
                    onClick={async () => {
                      const newToken = await createToken({
                        name: tokenName,
                      });
                      setCreatedTokens([...createdTokens, newToken]);
                      setIsDialogOpen(false);
                    }}
                  />
                }
              />
            </Dialog>
          </div>
          <Divider className="mt-10 mb-4" />
          <HTMLTable striped bordered>
            <thead>
              <tr>
                <th>{t('name')}</th>
                <th>{t('tokenValue')}</th>
                <th>{t('createdAt')}</th>
                <th> </th>
              </tr>
            </thead>
            <tbody>
              {createdTokens
                .sort(
                  (a, b) => Date.parse(b.createdAt) - Date.parse(a.createdAt)
                )
                .map((token) => (
                  <tr key={token.id}>
                    <td>{token.name}</td>
                    <td className="flex justify-center">
                      <Button
                        icon="clipboard"
                        minimal
                        onClick={() => {
                          navigator.clipboard.writeText(token.token);
                        }}
                      >
                        {t('copyValue')}
                      </Button>
                    </td>
                    <td>{new Date(token.createdAt).toLocaleString()}</td>
                    <td>
                      <Button intent="danger" icon="trash" small>
                        {t('delete')}
                      </Button>
                    </td>
                  </tr>
                ))}
              <tr>
                <td>Blueprint</td>
                <td>CSS framework and UI toolkit</td>
                <td> </td>
                <td>
                  <Button intent="danger" icon="trash" small>
                    {t('delete')}
                  </Button>
                </td>
              </tr>
              <tr>
                <td>TSLint</td>
                <td>Static analysis linter for TypeScript</td>
                <td>TypeScript</td>
                <td>403</td>
              </tr>
              <tr>
                <td>Plottable</td>
                <td>Composable charting library built on top of D3</td>
                <td>SVG, TypeScript, D3</td>
                <td>737</td>
              </tr>
            </tbody>
          </HTMLTable>
        </div>
      </main>
    </div>
  );
};

export default Tokens;
